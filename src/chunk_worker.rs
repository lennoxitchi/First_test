use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use crate::chunk::{Chunk, ChunkPos, ChunkSnapshot};
use crate::chunk_mesh;
use crate::world_generator;

pub enum ChunkRequest {
    Generate {
        position: ChunkPos,
        neighbors: [Option<ChunkSnapshot>; 6],
    },

    Remesh {
        position: ChunkPos,
        chunk: ChunkSnapshot,
        neighbors: [Option<ChunkSnapshot>; 6],
    },
}

pub struct GeneratedChunk {
    pub position: ChunkPos,
    pub chunk: Option<Chunk>,
    pub mesh: chunk_mesh::ChunkMesh,
}

pub struct ChunkWorker {
    sender: Sender<ChunkRequest>,
    receiver: Receiver<GeneratedChunk>,
}

impl ChunkWorker {
    pub fn new() -> Self {
        let (request_sender, request_receiver) =
            mpsc::channel::<ChunkRequest>();

        let (result_sender, result_receiver) =
            mpsc::channel::<GeneratedChunk>();

        thread::spawn(move || {
            while let Ok(request) = request_receiver.recv() {
                match request {
                    ChunkRequest::Generate {
                        position,
                        neighbors,
                    } => {
                        let chunk =
                            world_generator::generate_chunk(position);

                        let snapshot =
                            ChunkSnapshot::from(&chunk);

                        let mesh =
                            chunk_mesh::build_chunk_mesh(
                                &snapshot,
                                &neighbors,
                            );

                        if result_sender
                            .send(GeneratedChunk {
                                position,
                                chunk: Some(chunk),
                                mesh,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }

                    ChunkRequest::Remesh {
                        position,
                        chunk,
                        neighbors,
                    } => {
                        let mesh =
                            chunk_mesh::build_chunk_mesh(
                                &chunk,
                                &neighbors,
                            );

                        if result_sender
                            .send(GeneratedChunk {
                                position,
                                chunk: None,
                                mesh,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                }
            }
        });

        Self {
            sender: request_sender,
            receiver: result_receiver,
        }
    }

    pub fn request(
        &self,
        position: ChunkPos,
        neighbors: [Option<ChunkSnapshot>; 6],
    ) {
        let _ = self.sender.send(
            ChunkRequest::Generate {
                position,
                neighbors,
            }
        );
    }

    pub fn remesh(
        &self,
        position: ChunkPos,
        chunk: ChunkSnapshot,
        neighbors: [Option<ChunkSnapshot>; 6],
    ) {
        let _ = self.sender.send(
            ChunkRequest::Remesh {
                position,
                chunk,
                neighbors,
            }
        );
    }

    pub fn try_receive(&self) -> Option<GeneratedChunk> {
        self.receiver.try_recv().ok()
    }
}