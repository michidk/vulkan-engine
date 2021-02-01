use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::mesh::{Mesh, Submesh};

#[derive(Debug, Default)]
struct State {
    current_submesh: Option<Submesh>,
}

// parses wavefront obj
pub fn parse(filepath: &str) {
    if let Ok(lines) = read_lines(filepath) {
        log::info!("Loading mesh: {}", filepath);
        for line in lines {
            if let Ok(line) = line {
                println!("Parsing: {}", line);

                let mut mesh: Mesh = Mesh::default();
                let mut state: State = State::default();

                if let Some((token, value)) = line.split_once(' ') {
                    parse_token(token, value, &mut mesh, &mut state);
                }
            }
        }
    }
}

fn parse_token(token: &str, value: &str, mesh: &mut Mesh, state: &mut State) {
    match token {
        // comment
        "#" => log::info!("Comment: {:?}", value.get(2..).unwrap_or("")),
        // material
        "mtllib" => log::warn!(".mtl materials are not implemented yet"),
        // name
        "o" => {
            mesh.name = value.into();
        }
        // group
        "g" => {
            if let Some(x) = state.current_submesh.take() {
                mesh.submeshes.push(x);
            }
            state.current_submesh = Some(Submesh {
                name: value.into(),
                ..Default::default()
            });
        }
        // vertex
        "v" => {}
        // material
        "usemtl" => {}
        // smothing groups
        "s" => {}
        _ => {}
    }
}

// fn parse_vec3 {

// }

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[cfg(test)]
mod test {
    use crate::mesh::Mesh;

    use super::{parse_token, State};

    #[test]
    fn test_parse_token() {
        let mut mesh: Mesh = Mesh::default();

        parse_token("o", "foo bar", &mut mesh, &mut State::default());

        assert_eq!(
            mesh,
            Mesh {
                name: String::from("foo bar"),
                ..Default::default()
            }
        );
    }
}
