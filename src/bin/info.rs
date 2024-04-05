use clap::Parser;
use dashmap::DashMap;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use md5::{compute, Digest};
use memmap::Mmap;
use rayon::prelude::*;
use std::{fmt::Display, fs::File, ops::Deref, path::PathBuf};

#[derive(Parser)]
struct Args {
	input: PathBuf,
	#[arg(default_value = "1")]
	amt: u64,
}

pub struct MD5Str<'a>(&'a str, Digest);

impl<'a> From<&'a str> for MD5Str<'a> {
	fn from(value: &'a str) -> Self {
		Self(value, compute(value))
	}
}

impl<'a> Eq for MD5Str<'a> {}
impl<'a> PartialEq for MD5Str<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.1 == other.1
	}
}

impl<'a> Deref for MD5Str<'a> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.0
	}
}

impl<'a> std::hash::Hash for MD5Str<'a> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state);
	}
}

impl<'a> Display for MD5Str<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

fn main() {
	let Args { input, amt } = Args::parse();
	let input = File::open(input).expect("Open input file");
	let input = unsafe { Mmap::map(&input).expect("Memmap input file") };
	let input = unsafe { std::str::from_utf8_unchecked(&input) };
	let (_, input) = input.split_once('\n').unwrap();
	let pb = ProgressBar::new(amt).with_style(
		ProgressStyle::with_template(
			"[{elapsed_precise} <- {eta_precise} at {per_sec}] {wide_bar}",
		)
		.unwrap(),
	);
	let ref_counts: DashMap<MD5Str<'_>, u32> = DashMap::new();
	input
		.par_lines()
		.progress_with(pb)
		.flat_map(|line| line.split_once(','))
		.for_each(|(_from, to)| {
			*ref_counts.entry(to.into()).or_default() += 1;
		});
	if let Some(it) = ref_counts.par_iter().max_by_key(|it| *it.value()) {
		let (max, amt) = it.pair();
		println!("Article \"{max}\" with {amt} references");
	};
}
