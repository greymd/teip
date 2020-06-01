/// Input stream is devided into Tokens
pub enum Token {
    Channel(String), // Unmatched string. Printed as it is.
    Piped,           // Matched string
    Solid(String),   // Matched string when -s (solid mode)
    EOF,             // End of file
}
