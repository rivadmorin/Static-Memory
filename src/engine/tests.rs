#[cfg(test)]
mod tests {
    use crate::engine::buffer::TextBuffer;

    #[test]
    fn test_buffer_basic() {
        let mut buf = TextBuffer::new();
        buf.push('a');
        buf.push('b');
        assert_eq!(buf.get_string(), "ab");
        buf.backspace();
        assert_eq!(buf.get_string(), "a");
    }

    #[test]
    fn test_buffer_clear() {
        let mut buf = TextBuffer::new();
        buf.push('a');
        buf.clear();
        assert!(buf.is_empty());
    }
}
