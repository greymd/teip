/// Input stream is devided into multiple Chunks
pub enum Chunk {
    Keep(String),   // a string under masking tape. Printed as is.
    Hole,           // A hole on the masking tape. The string in the hole being processed other thread.
    SHole(String),  // Solid hole. A hole and string in this hole. Enabled with -s (solid mode)
    EOF,            // End of file
}

pub enum ChunkGroup {
    Keep,   // a string under masking tape. Printed as is.
    Hole,   // A hole on the masking tape. The string in the hole being processed other thread.
}
