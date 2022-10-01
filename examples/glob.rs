use glob::glob;

fn main() {
    let files = glob("fixtures/db/**/*.sql")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|p| {
            p.components().all(|c| {
                c.as_os_str()
                    .to_str()
                    .map(|s| !s.starts_with('_'))
                    .unwrap_or(true)
            })
        })
        .collect::<Vec<_>>();
    for file in files {
        println!("{:?}", file);
    }
}
