use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::Write;
use std::process::{Command, Stdio};

static CMD: &'static str = "./target/release/teip";

fn character_double(lap: usize) {
    let mut child = Command::new(CMD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null()) // comment out to check output.
        .args(&["-c", "1-3,6-8", "sed", "s/./A/"])
        .spawn()
        .expect("Failed to swapn process");
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Child process stdin has not been captured!")
            .unwrap();
        stdin
            .write_all("@@@@@@@@@@\n".repeat(lap).as_bytes())
            .unwrap();
    }
    let _ = child.wait_with_output();
}

fn standard_regex_double(lap: usize) {
    let mut child = Command::new(CMD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null()) // comment out to check output.
        .args(&["-r", "\\d+", "sed", "s/./@/"])
        .spawn()
        .expect("Failed to swapn process");
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Child process stdin has not been captured!")
            .unwrap();
        stdin
            .write_all("ABC123DEF456\n".repeat(lap).as_bytes())
            .unwrap();
    }
    let _ = child.wait_with_output();
}

fn pcre_double(lap: usize) {
    let mut child = Command::new(CMD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null()) // comment out to check output.
        .args(&["-P", "\\d+", "sed", "s/./@/"])
        .spawn()
        .expect("Failed to swapn process");
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Child process stdin has not been captured!")
            .unwrap();
        stdin
            .write_all("ABC123DEF456\n".repeat(lap).as_bytes())
            .unwrap();
    }
    let _ = child.wait_with_output();
}



fn field_double(lap: usize) {
    let mut child = Command::new(CMD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null()) // comment out to check output.
        .args(&["-d", ",", "-f", "2,4", "sed", "s/./@/"])
        .spawn()
        .expect("Failed to swapn process");
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Child process stdin has not been captured!")
            .unwrap();
        stdin
            .write_all("AAA,BBB,CCC,DDD,EEE\n".repeat(lap).as_bytes())
            .unwrap();
    }
    let _ = child.wait_with_output();
}

fn field_regex_double(lap: usize) {
    let mut child = Command::new(CMD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null()) // comment out to check output.
        .args(&["-f", "2,4", "sed", "s/./@/"])
        .spawn()
        .expect("Failed to swapn process");
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Child process stdin has not been captured!")
            .unwrap();
        stdin
            .write_all("AAA BBB CCC DDD EEE\n".repeat(lap).as_bytes())
            .unwrap();
    }
    let _ = child.wait_with_output();
}

fn solid_character_double(lap: usize) {
    let mut child = Command::new(CMD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null()) // comment out to check output.
        .args(&["-s", "-c", "1-3,6-8", "sed", "s/./A/"])
        .spawn()
        .expect("Failed to swapn process");
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Child process stdin has not been captured!")
            .unwrap();
        stdin
            .write_all("@@@@@@@@@@\n".repeat(lap).as_bytes())
            .unwrap();
    }
    let _ = child.wait_with_output();
}

fn solid_standard_regex_double(lap: usize) {
    let mut child = Command::new(CMD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null()) // comment out to check output.
        .args(&["-s", "-r", "\\d+", "sed", "s/./@/"])
        .spawn()
        .expect("Failed to swapn process");
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Child process stdin has not been captured!")
            .unwrap();
        stdin
            .write_all("ABC123DEF456\n".repeat(lap).as_bytes())
            .unwrap();
    }
    let _ = child.wait_with_output();
}

fn solid_field_double(lap: usize) {
    let mut child = Command::new(CMD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null()) // comment out to check output.
        .args(&["-s", "-d", ",", "-f", "2,4", "sed", "s/./@/"])
        .spawn()
        .expect("Failed to swapn process");
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Child process stdin has not been captured!")
            .unwrap();
        stdin
            .write_all("AAA,BBB,CCC,DDD,EEE\n".repeat(lap).as_bytes())
            .unwrap();
    }
    let _ = child.wait_with_output();
}

fn solid_field_regex_double(lap: usize) {
    let mut child = Command::new(CMD)
        .stdin(Stdio::piped())
        .stdout(Stdio::null()) // comment out to check output.
        .args(&["-s", "-f", "2,4", "sed", "s/./@/"])
        .spawn()
        .expect("Failed to swapn process");
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or("Child process stdin has not been captured!")
            .unwrap();
        stdin
            .write_all("AAA BBB CCC DDD EEE\n".repeat(lap).as_bytes())
            .unwrap();
    }
    let _ = child.wait_with_output();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("character_double 10000", |b| {
        b.iter(|| character_double(black_box(10000)))
    });
    c.bench_function("standard_regex_double 10000", |b| {
        b.iter(|| standard_regex_double(black_box(10000)))
    });
    c.bench_function("pcre_double 10000", |b| {
        b.iter(|| pcre_double(black_box(10000)))
    });
    c.bench_function("field_double 10000", |b| {
        b.iter(|| field_double(black_box(10000)))
    });
    c.bench_function("field_regex_double 10000", |b| {
        b.iter(|| field_regex_double(black_box(10000)))
    });
    c.bench_function("solid_character_double", |b| {
        b.iter(|| solid_character_double(black_box(100)))
    });
    c.bench_function("solid_standard_regex_double", |b| {
        b.iter(|| solid_standard_regex_double(black_box(100)))
    });
    c.bench_function("solid_field_double", |b| {
        b.iter(|| solid_field_double(black_box(100)))
    });
    c.bench_function("solid_field_regex_double", |b| {
        b.iter(|| solid_field_regex_double(black_box(100)))
    });
}

fn custom_criterion() -> Criterion {
    Criterion::default().sample_size(10)
}

criterion_group! {
    name = benches;
    config = custom_criterion();
    targets = criterion_benchmark
}
criterion_main!(benches);
