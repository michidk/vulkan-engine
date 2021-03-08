use crystal::prelude::*;

use log::debug;
use ve_format::mesh;

use super::parser::ParserError;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct ObjVertex {
    pub(crate) position: [f32; 3],
    pub(crate) color: Option<[f32; 3]>,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct ObjFaceIndex {
    pub(crate) vert_i: usize,
    pub(crate) uv_i: Option<usize>,
    pub(crate) normal_i: Option<usize>,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct ObjFace {
    pub(crate) face_i: Vec<ObjFaceIndex>,
}

#[derive(Debug, Default)]
pub(crate) struct ObjSubmesh {
    pub(crate) name: Option<String>,
    pub(crate) faces: Vec<ObjFace>,
}

#[derive(Debug, Default)]
pub(crate) struct ObjMeshData {
    pub(crate) name: Option<String>,
    pub(crate) submeshes: Vec<ObjSubmesh>,
    pub(crate) vertices: Vec<ObjVertex>,
    pub(crate) uvs: Vec<[f32; 2]>,
    pub(crate) normals: Vec<[f32; 3]>,
}

#[derive(Debug, Default)]
pub(crate) struct ObjMeshBuilder {
    pub(crate) mesh: ObjMeshData,
    pub(crate) curr_submesh: ObjSubmesh,
}

impl ObjMeshBuilder {
    pub(crate) fn set_group(&mut self, name: &str) {
        if self.curr_submesh.faces.is_empty() {
            self.curr_submesh.name = if name.is_empty() {
                None
            } else {
                Some(name.into())
            };
        } else {
            let mut fg = ObjSubmesh {
                name: if name.is_empty() {
                    None
                } else {
                    Some(name.into())
                },
                ..ObjSubmesh::default()
            };
            std::mem::swap(&mut fg, &mut self.curr_submesh);
            self.mesh.submeshes.push(fg);
        }
    }

    pub(crate) fn push_vertex(&mut self, vertex: ObjVertex) {
        self.mesh.vertices.push(vertex);
    }

    pub(crate) fn push_uv(&mut self, uv: [f32; 2]) {
        self.mesh.uvs.push(uv);
    }

    pub(crate) fn push_normal(&mut self, normal: [f32; 3]) {
        self.mesh.normals.push(normal);
    }

    pub(crate) fn push_face(&mut self, face: ObjFace) {
        self.curr_submesh.faces.push(face);
    }

    pub(crate) fn build_mesh(self) -> Result<mesh::MeshData, ParserError> {
        let mut mesh = self.mesh;
        let (uvs, normals) = (mesh.uvs, mesh.normals);
        mesh.submeshes.push(self.curr_submesh); // push the last group/submesh

        // dont do, only create for refereced verticies
        let mut vertices: Vec<mesh::Vertex> = Vec::new();

        let mut submeshes: Vec<mesh::Submesh> = Vec::new();
        for submesh in mesh.submeshes {
            let mut faces: Vec<mesh::Face> = Vec::new();
            for mut face in submesh.faces {
                let mut local_face_idx: Vec<usize> = Vec::new();

                // set uv and normal if they appear on a face
                // also duplicate the vertex if it was already defined with another uv or normal
                for i in 0..=1 {
                    let obj_vert = &mesh.vertices[face.face_i[i].vert_i - 1];

                    local_face_idx.push(Self::find_vertex(
                        i,
                        &uvs,
                        &normals,
                        &mut face,
                        &obj_vert,
                        &mut vertices,
                    ));
                }

                // triangulate polygons for convex shapes (we might find faces which have more than three indexes)
                for i in 2..face.face_i.len() {
                    let obj_vert = &mesh.vertices[face.face_i[i].vert_i - 1];

                    let idx =
                        Self::find_vertex(i, &uvs, &normals, &mut face, &obj_vert, &mut vertices);

                    debug!(
                        "Create triangle between {}, {}, {}",
                        local_face_idx[0],
                        local_face_idx[i - 1],
                        idx
                    );

                    faces.push(mesh::Face {
                        indices: [
                            (local_face_idx[0]) as u32,
                            (local_face_idx[i - 1]) as u32,
                            (idx) as u32,
                        ],
                    });

                    local_face_idx.push(idx);
                }
            }
            submeshes.push(mesh::Submesh { faces })
        }

        Ok(mesh::MeshData {
            vertices,
            submeshes,
        })
    }

    fn find_vertex(
        i: usize,
        uvs: &Vec<[f32; 2]>,
        normals: &Vec<[f32; 3]>,
        face: &mut ObjFace,
        obj_vert: &ObjVertex,
        vertices: &mut Vec<mesh::Vertex>,
    ) -> usize {
        // convert position to vec
        let position: Vec3<f32> = obj_vert.position.into();

        // get color values or default as vec
        let color: Vec3<f32> = obj_vert.color.unwrap_or_else(|| [0.0, 0.0, 0.0]).into();

        // get uv values from current face index or default as vec
        let uv: Vec2<f32> = face.face_i[i]
            .uv_i
            .map(|x| uvs[x - 1])
            .unwrap_or_else(|| [0.0, 0.0])
            .into();

        // get normal values from current face index or default as vec
        let mut normal: Vec3<f32> = face.face_i[i]
            .normal_i
            .map(|x| normals[x - 1])
            .unwrap_or_else(|| [0.0, 0.0, 0.0])
            .into();

        *normal.x_mut() = -*normal.x_mut();
        *normal.z_mut() = -*normal.z_mut();

        // search if vertex already exists, will result in O(n*log(n))
        let mut potential_vert_idx: Option<usize> = None;
        for (idx, existing_vertex) in vertices.iter().enumerate() {
            if existing_vertex.position == position
                && existing_vertex.color == color
                && existing_vertex.uv == uv
                && existing_vertex.normal == normal
            {
                potential_vert_idx = Some(idx);
                break;
            }
        }

        let vert_idx: usize;
        // create new vertex if not already existing
        if potential_vert_idx.is_none() {
            let vertex = mesh::Vertex {
                position,
                color,
                uv,
                normal,
            };
            vertices.push(vertex);
            vert_idx = vertices.len() - 1;
        } else {
            vert_idx = potential_vert_idx.unwrap();
        }

        vert_idx
    }
}
