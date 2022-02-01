use std::fs;
use std::io::{self, BufRead};
use std::{num, path::Path};

use log::debug;

use super::{builder::*, meta::ObjMeta};

#[derive(thiserror::Error, Debug)]
pub enum ParserError {
    #[error("Failed to parse float.")]
    ParseFloat(#[from] num::ParseFloatError),
    #[error("Failed to parse integer.")]
    ParseInt(#[from] num::ParseIntError),
    #[error("Failed to parse model.")]
    Io(#[from] io::Error),
    #[error("Failed to parse face.")]
    ParseFace,
}

// parses wavefront obj (https://en.wikipedia.org/wiki/Wavefront_.obj_file)
// the implementation is very forgiving and should work with most .obj files
pub(crate) fn parse(filepath: &Path, meta: ObjMeta) -> Result<ObjMeshBuilder, ParserError> {
    let mut builder: ObjMeshBuilder = ObjMeshBuilder {
        mesh: ObjMeshData::default(),
        meta,
        ..Default::default()
    };

    let lines = read_lines(filepath)?;
    log::info!("Loading mesh: {}", filepath.display());

    for line in lines.flatten() {
        if line.is_empty() {
            continue;
        }

        debug!("Parsing: \"{}\"", line);

        if let Some((token, value)) = line.replace("  ", " ").split_once(' ') {
            parse_token(token, value.trim(), &mut builder)?;
        }
    }

    Ok(builder)
}

fn parse_token(token: &str, value: &str, builder: &mut ObjMeshBuilder) -> Result<(), ParserError> {
    match token {
        // comment
        "#" => log::info!("Comment: {:?}", value),
        // material
        "mtllib" => log::warn!(".mtl materials are not implemented yet"),
        // name
        "o" => {
            builder.mesh.name = if value.is_empty() {
                None
            } else {
                Some(value.into())
            };
        }
        // group (submesh)
        "g" => builder.set_group(value),
        // vertex
        "v" => builder.push_vertex(parse_vertex(value)?),
        // texture coordinates
        "vt" => builder.push_uv(parse_uv(value)?),
        // vertex normals
        "vn" => builder.push_normal(parse_normal(value)?),
        // parameter space vertices
        "vp" => log::warn!("Parameter space vertices not supported. Ignoring."),
        "f" => builder.push_face(parse_face(value)?),
        // material
        "usemtl" => log::warn!("Materials not yet supported. Ignoring."),
        // smothing groups
        "s" => log::warn!("Smothing groups not supported. Ignoring."),
        "" => log::warn!("Found space. Ignoring."),
        _ => log::error!("Found invalid token: \"{}\"", token),
    };

    Ok(())
}

fn parse_vertex(value: &str) -> Result<ObjVertex, num::ParseFloatError> {
    let vec = parse_numbers(value)?;

    // check for colors
    let mut color = None;
    if vec.len() == 6 {
        color = Some([vec[3], vec[4], vec[5]]);
    }

    Ok(ObjVertex {
        position: [vec[0], vec[1], vec[2]],
        color,
    })
}

fn parse_normal(value: &str) -> Result<[f32; 3], num::ParseFloatError> {
    let numbers = parse_numbers(value)?;
    Ok([numbers[0], numbers[1], numbers[2]])
}

fn parse_uv(value: &str) -> Result<[f32; 2], num::ParseFloatError> {
    let numbers = parse_numbers(value)?;
    Ok([numbers[0], numbers[1]])
}

// parses numbers seperated by spaces
fn parse_numbers(value: &str) -> Result<Vec<f32>, num::ParseFloatError> {
    value
        .split(' ')
        .map(|x| x.parse())
        .collect::<Result<_, _>>()
}

// parses triples/face indexes sperated by spaces, which are itself seperated by dashes
fn parse_face(value: &str) -> Result<ObjFace, ParserError> {
    let face_indexes: Result<Vec<ObjFaceIndex>, _> = value
        .split(' ')
        .map(parse_face_index)
        .collect::<Result<_, _>>();
    Ok(ObjFace {
        face_i: face_indexes?,
    })
}

// parses a single face index seperated by dashes
fn parse_face_index(value: &str) -> Result<ObjFaceIndex, ParserError> {
    let triplet = parse_triplet(value)?;

    Ok(ObjFaceIndex {
        vert_i: triplet[0].ok_or(ParserError::ParseFace)?,
        uv_i: triplet[1],
        normal_i: triplet[2],
    })
}

// parse a triplet seperated by dashes
fn parse_triplet(value: &str) -> Result<Vec<Option<usize>>, num::ParseIntError> {
    let mut ret = vec![None; 3];

    for (a, b) in ret.iter_mut().zip(value.split('/')) {
        *a = if b.is_empty() { None } else { Some(b.parse()?) }
    }

    Ok(ret)
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::num::ParseIntError;

    #[test]
    fn test_parse_token() -> Result<(), ParserError> {
        let mut builder: ObjMeshBuilder = ObjMeshBuilder {
            meta: ObjMeta {
                flip_axis: [false, false, false],
                ..Default::default()
            },
            ..Default::default()
        };

        parse_token("o", "foo bar", &mut builder)?;
        parse_token("v", "1 2 3", &mut builder)?;
        parse_token("v", "4 5 6", &mut builder)?;
        parse_token("f", "1 2 2", &mut builder)?;
        parse_token("g", "new group", &mut builder)?;

        assert_eq!(builder.mesh.name, Some("foo bar".into()));
        assert_eq!(
            builder.mesh.vertices,
            vec![
                ObjVertex {
                    position: [1.0, 2.0, 3.0],
                    ..ObjVertex::default()
                },
                ObjVertex {
                    position: [4.0, 5.0, 6.0],
                    ..ObjVertex::default()
                }
            ]
        );

        Ok(())
    }

    #[test]
    fn test_parse_vertex() -> Result<(), ParserError> {
        assert_eq!(
            parse_vertex("1 1 1 1 2 3")?,
            ObjVertex {
                position: [1.0, 1.0, 1.0],
                color: Some([1.0, 2.0, 3.0])
            }
        );
        assert_eq!(
            parse_vertex("1 1 1 1")?,
            ObjVertex {
                position: [1.0, 1.0, 1.0],
                ..ObjVertex::default()
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_face() -> Result<(), ParserError> {
        assert_eq!(
            parse_face("1 2/2 3/2/1 5//2")?,
            ObjFace {
                face_i: vec![
                    ObjFaceIndex {
                        vert_i: 1,
                        ..ObjFaceIndex::default()
                    },
                    ObjFaceIndex {
                        vert_i: 2,
                        uv_i: Some(2),
                        ..ObjFaceIndex::default()
                    },
                    ObjFaceIndex {
                        vert_i: 3,
                        uv_i: Some(2),
                        normal_i: Some(1),
                    },
                    ObjFaceIndex {
                        vert_i: 5,
                        normal_i: Some(2),
                        ..ObjFaceIndex::default()
                    }
                ]
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_triplet() -> Result<(), ParseIntError> {
        assert_eq!(parse_triplet("1")?, &[Some(1), None, None]);
        assert_eq!(parse_triplet("1/3")?, &[Some(1), Some(3), None]);
        assert_eq!(parse_triplet("1/2/3")?, &[Some(1), Some(2), Some(3)]);
        assert_eq!(parse_triplet("1//3")?, &[Some(1), None, Some(3)]);

        Ok(())
    }
}
