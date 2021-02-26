use std::fs;
use std::io::{self, BufRead};
use std::{num, path::Path};

use crate::scene::model::mesh;

#[derive(Debug, Default, PartialEq)]
pub struct Vertex {
    position: [f32; 3],
    color: Option<[f32; 3]>,
}

#[derive(Debug, Default, PartialEq)]
pub struct FaceIndex {
    vert_i: usize,
    uv_i: Option<usize>,
    normal_i: Option<usize>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Face {
    face_i: Vec<FaceIndex>,
}

#[derive(Debug, Default)]
pub struct Submesh {
    name: Option<String>,
    faces: Vec<Face>,
}

#[derive(Debug, Default)]
pub struct Mesh {
    name: Option<String>,
    submeshes: Vec<Submesh>,
    vertices: Vec<Vertex>,
    uvs: Vec<[f32; 2]>,
    normals: Vec<[f32; 3]>,
}

#[derive(Debug, Default)]
pub struct MeshBuilder {
    mesh: Mesh,
    curr_submesh: Submesh,
}

impl MeshBuilder {
    fn set_group(&mut self, name: &str) {
        if self.curr_submesh.faces.is_empty() {
            self.curr_submesh.name = if name.is_empty() {
                None
            } else {
                Some(name.into())
            };
        } else {
            let mut fg = Submesh {
                name: if name.is_empty() {
                    None
                } else {
                    Some(name.into())
                },
                ..Submesh::default()
            };
            std::mem::swap(&mut fg, &mut self.curr_submesh);
            self.mesh.submeshes.push(fg);
        }
    }

    fn push_vertex(&mut self, vertex: Vertex) {
        self.mesh.vertices.push(vertex);
    }

    fn push_uv(&mut self, uv: [f32; 2]) {
        self.mesh.uvs.push(uv);
    }

    fn push_normal(&mut self, normal: [f32; 3]) {
        self.mesh.normals.push(normal);
    }

    fn push_face(&mut self, face: Face) {
        self.curr_submesh.faces.push(face);
    }

    pub fn build_mesh(self) -> Result<mesh::Mesh, ParserError> {
        let mut mesh = self.mesh;
        let (uvs, normals) = (mesh.uvs, mesh.normals);
        mesh.submeshes.push(self.curr_submesh); // push the last group/submesh

        let mut vertices: Vec<mesh::Vertex> = Vec::new();
        for vertex in mesh.vertices {
            vertices.push(mesh::Vertex {
                position: vertex.position.into(),
                color: vertex.color.map(Into::into),
                uv: None,
                normal: None,
            })
        }

        let mut submeshes: Vec<mesh::Submesh> = Vec::new();
        for submesh in mesh.submeshes {
            let mut faces: Vec<mesh::Face> = Vec::new();
            for mut face in submesh.faces {
                // set uv and normal if they appear on a face
                // also duplicate the vertex if it was already defined with another uv or normal
                for i in 0..=2 {
                    // let vertex = &self.mesh.vertices[face.face_i[i].vert_i];
                    let vertex = &mut vertices[face.face_i[i].vert_i - 1];

                    // get uv values from current face index
                    let uv = face.face_i[i].uv_i.map(|x| uvs[x - 1].into());

                    // get normal values from current face index
                    let normal = face.face_i[i].uv_i.map(|x| normals[x - 1].into());

                    // assign if empty, create new vertex if not (and assign face index to it)
                    if vertex.uv.is_none() && vertex.normal.is_none() {
                        vertex.uv = uv;
                        vertex.normal = normal;
                    }
                    if vertex.uv != uv || vertex.normal != normal {
                        let mut new_vertex = vertex.clone();
                        new_vertex.uv = uv;
                        new_vertex.normal = normal;
                        vertices.push(new_vertex);
                        face.face_i[i].vert_i = vertices.len();
                    }
                }

                // triangulate polygons for convex shapes (we might find faces which have more than three indexes)
                for i in 2..face.face_i.len() {
                    faces.push(mesh::Face {
                        indices: [
                            face.face_i[0].vert_i,
                            face.face_i[i - 1].vert_i,
                            face.face_i[i].vert_i,
                        ],
                    });
                    println!(
                        "Create triangle between {}, {}, {}",
                        face.face_i[0].vert_i,
                        face.face_i[i - 1].vert_i,
                        face.face_i[i].vert_i
                    );
                }
            }
            submeshes.push(mesh::Submesh {
                name: submesh.name,
                faces,
            })
        }

        Ok(mesh::Mesh {
            name: mesh.name,
            vertices,
            submeshes,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParserError {
    #[error("Failed to parse float.")]
    ParseFloatError(#[from] num::ParseFloatError),
    #[error("Failed to parse integer.")]
    ParseIntError(#[from] num::ParseIntError),
    #[error("Failed to parse model.")]
    IoError(#[from] io::Error),
    #[error("Failed to parse face.")]
    ParseFaceError,
}

// parses wavefront obj (https://en.wikipedia.org/wiki/Wavefront_.obj_file)
// the implementation is very forgiving and should work with most .obj files
pub fn parse(filepath: &str) -> Result<MeshBuilder, ParserError> {
    let mut builder: MeshBuilder = MeshBuilder {
        mesh: Mesh::default(),
        ..Default::default()
    };

    let lines = read_lines(filepath)?;
    log::info!("Loading mesh: {}", filepath);

    for line in lines.flatten() {
        println!("Parsing: {}", line);

        if let Some((token, value)) = line.split_once(' ') {
            parse_token(token, value, &mut builder)?;
        }
    }

    Ok(builder)
}

fn parse_token(token: &str, value: &str, builder: &mut MeshBuilder) -> Result<(), ParserError> {
    match token {
        // comment
        "#" => log::info!("Comment: {:?}", value.get(2..).unwrap_or("")),
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
        "usemtl" => todo!(),
        // smothing groups
        "s" => log::warn!("Smothing groups not supported. Ignoring."),
        _ => log::error!("Found invalid token: {}", token),
    };

    Ok(())
}

fn parse_vertex(value: &str) -> Result<Vertex, num::ParseFloatError> {
    let vec = parse_numbers(&value)?;

    // check for colors
    let mut color = None;
    if vec.len() == 6 {
        color = Some([vec[3], vec[4], vec[5]]);
    }

    Ok(Vertex {
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
fn parse_face(value: &str) -> Result<Face, ParserError> {
    let face_indexes: Result<Vec<FaceIndex>, _> = value
        .split(' ')
        .map(|x| parse_face_index(x))
        .collect::<Result<_, _>>();
    Ok(Face {
        face_i: face_indexes?,
    })
}

// parses a single face index seperated by dashes
fn parse_face_index(value: &str) -> Result<FaceIndex, ParserError> {
    let triplet = parse_triplet(value)?;

    Ok(FaceIndex {
        vert_i: triplet[0].ok_or(ParserError::ParseFaceError)?,
        uv_i: triplet[1],
        normal_i: triplet[2],
    })
}

// parse a triplet seperated by dashes
// TODO @Jonas: improve with .get
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
    use std::num::ParseIntError;

    use super::{
        parse_face, parse_token, parse_triplet, parse_vertex, Face, FaceIndex, MeshBuilder,
        ParserError, Vertex,
    };

    #[test]
    fn test_parse_token() -> Result<(), ParserError> {
        let mut builder: MeshBuilder = MeshBuilder::default();

        parse_token("o", "foo bar", &mut builder)?;
        parse_token("v", "1 2 3", &mut builder)?;
        parse_token("v", "4 5 6", &mut builder)?;
        parse_token("f", "1 2 2", &mut builder)?;
        parse_token("g", "new group", &mut builder)?;

        assert_eq!(builder.mesh.name, Some("foo bar".into()));
        assert_eq!(
            builder.mesh.vertices,
            vec![
                Vertex {
                    position: [1.0, 2.0, 3.0],
                    ..Vertex::default()
                },
                Vertex {
                    position: [4.0, 5.0, 6.0],
                    ..Vertex::default()
                }
            ]
        );

        Ok(())
    }

    #[test]
    fn test_parse_vertex() -> Result<(), ParserError> {
        assert_eq!(
            parse_vertex("1 1 1 1 2 3")?,
            Vertex {
                position: [1.0, 1.0, 1.0],
                color: Some([1.0, 2.0, 3.0])
            }
        );
        assert_eq!(
            parse_vertex("1 1 1 1")?,
            Vertex {
                position: [1.0, 1.0, 1.0],
                ..Vertex::default()
            }
        );

        Ok(())
    }

    #[test]
    fn test_parse_face() -> Result<(), ParserError> {
        assert_eq!(
            parse_face("1 2/2 3/2/1 5//2")?,
            Face {
                face_i: vec![
                    FaceIndex {
                        vert_i: 1,
                        ..FaceIndex::default()
                    },
                    FaceIndex {
                        vert_i: 2,
                        uv_i: Some(2),
                        ..FaceIndex::default()
                    },
                    FaceIndex {
                        vert_i: 3,
                        uv_i: Some(2),
                        normal_i: Some(1),
                    },
                    FaceIndex {
                        vert_i: 5,
                        normal_i: Some(2),
                        ..FaceIndex::default()
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
