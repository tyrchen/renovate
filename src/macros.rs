#[macro_export]
macro_rules! map_insert_schema {
    ($map:expr, $item:ident) => {
        $map.entry($item.id.schema.clone())
            .or_insert(Default::default())
            .insert($item.id.name.clone(), $item);
    };
}

#[macro_export]
macro_rules! map_insert_relation {
    ($map:expr, $item:ident) => {
        $map.entry($item.id.schema_id.clone())
            .or_insert(Default::default())
            .insert($item.id.name.clone(), $item);
    };
}

#[macro_export]
macro_rules! schema_diff {
    ($local:expr, $remote:expr, $migrations:ident, $t:ty) => {
        let keys: HashSet<_> = $local.keys().collect();
        let other_keys: HashSet<_> = $remote.keys().collect();

        // process intersection
        let intersection = keys.intersection(&other_keys);
        for key in intersection {
            let local = $local.get(*key).unwrap();
            let remote = $remote.get(*key).unwrap();
            let keys: HashSet<_> = local.keys().collect();
            let other_keys: HashSet<_> = remote.keys().collect();
            let added = keys.difference(&other_keys);
            for key in added {
                let v = local.get(*key).unwrap().clone();
                let id = v.id();
                let diff = NodeDiff::with_new(v);
                if atty::is(atty::Stream::Stdout) {
                    println!("{} {} is added:\n{}", stringify!($t), id, diff.diff);
                }
                $migrations.extend(diff.plan()?);
            }
            let removed = other_keys.difference(&keys);
            for key in removed {
                let v = remote.get(*key).unwrap().clone();
                let id = v.id();
                let diff = NodeDiff::with_old(v);
                if atty::is(atty::Stream::Stdout) {
                    println!("{} {} is removed:\n{}", stringify!($t), id, diff.diff);
                }
                $migrations.extend(diff.plan()?);
            }
            let intersection = keys.intersection(&other_keys);
            for key in intersection {
                let local: $t = local.get(*key).unwrap().to_string().parse()?;
                let remote: $t = remote.get(*key).unwrap().to_string().parse()?;

                let diff = remote.diff(&local)?;
                if let Some(diff) = diff {
                    if atty::is(atty::Stream::Stdout) {
                        println!(
                            "{} {} is changed:\n\n{}",
                            stringify!($t),
                            local.id(),
                            diff.diff
                        );
                    }
                    $migrations.extend(diff.plan()?);
                }
            }
        }

        // process added
        let added = keys.difference(&other_keys);
        for key in added {
            $migrations.push(format!("CREATE SCHEMA {}", key));
            for (_name, item) in $local.get(*key).unwrap() {
                let diff = NodeDiff::with_new(item.clone());
                if atty::is(atty::Stream::Stdout) {
                    println!("{} {} is added:\n{}", stringify!($t), item.id(), diff.diff);
                }
                $migrations.extend(diff.plan()?);
            }
        }

        // process removed
        let removed = other_keys.difference(&keys);
        for key in removed {
            for (_name, item) in $remote.get(*key).unwrap() {
                let diff = NodeDiff::with_old(item.clone());
                if atty::is(atty::Stream::Stdout) {
                    println!(
                        "{} {} is removed:\n{}",
                        stringify!($t),
                        item.id(),
                        diff.diff
                    );
                }
                $migrations.extend(diff.plan()?);
            }
            $migrations.push(format!("DROP SCHEMA {}", key));
        }
    };
}
