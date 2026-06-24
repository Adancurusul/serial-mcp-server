#[cfg(test)]
mod tests {
    use super::super::types::{decode_data, encode_data, SetControlLinesArgs};

    #[test]
    fn test_decode_utf8() {
        let result = decode_data("Hello, 世界!", "utf8").unwrap();
        assert_eq!(result, "Hello, 世界!".as_bytes());
    }

    #[test]
    fn test_encode_utf8() {
        let data = "Hello, 世界!".as_bytes();
        let result = encode_data(data, "utf8").unwrap();
        assert_eq!(result, "Hello, 世界!");
    }

    #[test]
    fn test_decode_hex() {
        // Test basic hex
        let result = decode_data("48656c6c6f", "hex").unwrap();
        assert_eq!(result, b"Hello");

        // Test hex with spaces
        let result = decode_data("48 65 6c 6c 6f", "hex").unwrap();
        assert_eq!(result, b"Hello");

        // Test uppercase hex
        let result = decode_data("48656C6C6F", "hex").unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_encode_hex() {
        let data = b"Hello";
        let result = encode_data(data, "hex").unwrap();
        assert_eq!(result, "48 65 6c 6c 6f");
    }

    #[test]
    fn test_decode_hex_invalid() {
        // Odd length
        assert!(decode_data("48656c6c6", "hex").is_err());
        
        // Invalid characters
        assert!(decode_data("48656cXY", "hex").is_err());
    }

    #[test]
    fn test_decode_base64() {
        let result = decode_data("SGVsbG8gV29ybGQ=", "base64").unwrap();
        assert_eq!(result, b"Hello World");

        // Test without padding
        let result = decode_data("SGVsbG8gV29ybGQ", "base64").unwrap();
        assert_eq!(result, b"Hello World");
    }

    #[test]
    fn test_encode_base64() {
        let data = b"Hello World";
        let result = encode_data(data, "base64").unwrap();
        assert_eq!(result, "SGVsbG8gV29ybGQ=");
    }

    #[test]
    fn test_decode_base64_invalid() {
        assert!(decode_data("SGVsbG8gV29ybGQ===", "base64").is_err());
        assert!(decode_data("Invalid@Base64", "base64").is_err());
    }

    #[test]
    fn test_unsupported_encoding() {
        assert!(decode_data("test", "unknown").is_err());
        assert!(encode_data(b"test", "unknown").is_err());
    }

    #[test]
    fn test_encode_utf8_invalid() {
        // Invalid UTF-8 sequence
        let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
        assert!(encode_data(&invalid_utf8, "utf8").is_err());
    }

    #[test]
    fn test_roundtrip_encodings() {
        let test_data = b"Hello, World! 123 \x00\xFF";
        
        // Test hex roundtrip
        let hex_encoded = encode_data(test_data, "hex").unwrap();
        let hex_decoded = decode_data(&hex_encoded, "hex").unwrap();
        assert_eq!(test_data, hex_decoded.as_slice());

        // Test base64 roundtrip
        let b64_encoded = encode_data(test_data, "base64").unwrap();
        let b64_decoded = decode_data(&b64_encoded, "base64").unwrap();
        assert_eq!(test_data, b64_decoded.as_slice());
    }

    #[test]
    fn test_set_control_lines_requires_at_least_one_line() {
        let args = SetControlLinesArgs {
            connection_id: "conn-1".to_string(),
            rts: None,
            dtr: None,
        };
        assert!(!args.has_line_update());

        let args = SetControlLinesArgs {
            connection_id: "conn-1".to_string(),
            rts: Some(true),
            dtr: None,
        };
        assert!(args.has_line_update());

        let args = SetControlLinesArgs {
            connection_id: "conn-1".to_string(),
            rts: None,
            dtr: Some(false),
        };
        assert!(args.has_line_update());
    }

    #[test]
    fn test_set_control_lines_deserializes_optional_lines() {
        let args: SetControlLinesArgs =
            serde_json::from_str(r#"{"connection_id":"conn-1"}"#).unwrap();
        assert_eq!(args.connection_id, "conn-1");
        assert_eq!(args.rts, None);
        assert_eq!(args.dtr, None);
        assert!(!args.has_line_update());

        let args: SetControlLinesArgs =
            serde_json::from_str(r#"{"connection_id":"conn-1","rts":true,"dtr":false}"#)
                .unwrap();
        assert_eq!(args.rts, Some(true));
        assert_eq!(args.dtr, Some(false));
        assert!(args.has_line_update());
    }
}
