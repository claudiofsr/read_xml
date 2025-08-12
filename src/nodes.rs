use crate::MyResult;
use quick_xml::{
    events::{BytesStart, Event},
    reader::Reader,
};
use std::{collections::HashMap, path::PathBuf};

// Autor: Adrian Macal
// https://amacal.medium.com/learn-rust-parsing-big-xml-files-67ec923f6977
// https://github.com/amacal/learning-rust/tree/reading-big-xml

fn increment_counters(seen: &mut HashMap<String, usize>, key: String) {
    if let Some(value) = seen.get_mut(&key) {
        (*value) += 1;
    } else {
        seen.insert(key, 1);
    }
}

fn process_attributes(seen: &mut HashMap<String, usize>, path: &mut Vec<String>, node: BytesStart) {
    path.push("@attribute".into());

    for attribute in node.attributes() {
        path.push(format!(
            "{:?}",
            String::from_utf8(attribute.unwrap().key.0.to_vec()).unwrap()
        ));
        increment_counters(seen, path.join(" / "));
        path.pop();
    }

    path.pop();
}

fn print_results(seen: &HashMap<String, usize>, counter: &usize) {
    let mut keys: Vec<&String> = seen.keys().collect();
    keys.sort();

    for item in keys {
        println!("{}: {:?}", item, seen.get(item).unwrap());
    }

    println!("Found not considered nodes: {counter}");
}

#[tokio::main]
pub async fn print_nodes(xml_path: &PathBuf) -> MyResult<()> {
    let mut reader = Reader::from_file(xml_path).unwrap();

    // path, which works like a stack showing the way from the root to the current node
    // seen, which is a dictionary that keeps track of all the paths we have already come across

    let mut path = Vec::new();
    let mut seen = HashMap::new();

    let mut buffer = Vec::new();
    let mut counter = 0;

    loop {
        match reader.read_event_into(&mut buffer) {
            Err(error) => break println!("{error}"),
            Ok(Event::Eof) => break println!("Completed."),
            Ok(Event::Start(node)) => {
                path.push(format!(
                    "{:?}",
                    String::from_utf8(node.name().0.to_vec()).unwrap()
                ));
                increment_counters(&mut seen, path.join(" / "));
                process_attributes(&mut seen, &mut path, node);
            }
            Ok(Event::End(_)) => {
                path.pop();
            }
            Ok(Event::Text(e)) => {
                //path.push("@text".into());
                path.push(e.decode().expect("Invalid UTF-8!").into_owned());
                increment_counters(&mut seen, path.join(" / "));
                path.pop();
            }
            Ok(Event::Empty(node)) => {
                path.push(format!(
                    "{:?}",
                    String::from_utf8(node.name().0.to_vec()).unwrap()
                ));
                increment_counters(&mut seen, path.join(" / "));
                process_attributes(&mut seen, &mut path, node);
                path.pop();
            }
            Ok(_) => counter += 1,
        }

        buffer.clear();
    }

    print_results(&seen, &counter);
    Ok(())
}
