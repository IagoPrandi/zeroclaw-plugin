#[must_use]
pub fn parameters_schema() -> String {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "required": ["source", "cluster"],
        "properties": {
            "source": {
                "type": "object",
                "description": "Discriminated source object. Use type=serialized with transaction_base64, or type=confirmed with signature. Never encode this object as a JSON string.",
                "additionalProperties": false,
                "required": ["type"],
                "properties": {
                    "type": {"type": "string", "enum": ["serialized", "confirmed"]},
                    "transaction_base64": {"type": "string", "minLength": 1, "maxLength": 8192},
                    "signature": {"type": "string", "minLength": 80, "maxLength": 100}
                }
            },
            "cluster": {"type": "string", "enum": ["mainnet-beta", "devnet", "testnet"]},
            "observed_wallets": {
                "type": "array", "maxItems": 10, "uniqueItems": true,
                "items": {"type": "string", "minLength": 32, "maxLength": 44}
            },
            "expected_intent": {"$ref": "#/$defs/expectedIntent"},
            "output_language": {"type": "string", "enum": ["en", "pt-BR"], "default": "en"}
        },
        "$defs": {
            "expectedIntent": {
                "type": "object", "additionalProperties": false,
                "properties": {
                    "description": {"type": "string", "maxLength": 500},
                    "allowed_programs": {
                        "type": "array", "maxItems": 20, "uniqueItems": true,
                        "items": {"type": "string"}
                    },
                    "allowed_recipients": {
                        "type": "array", "maxItems": 20, "uniqueItems": true,
                        "items": {"type": "string"}
                    },
                    "max_sol_out_lamports": {"type": "string", "pattern": "^[0-9]+$"},
                    "token_limits": {
                        "type": "array", "maxItems": 20,
                        "items": {
                            "type": "object", "additionalProperties": false, "required": ["mint"],
                            "properties": {
                                "mint": {"type": "string"},
                                "max_out_raw": {"type": "string", "pattern": "^[0-9]+$"},
                                "min_in_raw": {"type": "string", "pattern": "^[0-9]+$"}
                            }
                        }
                    }
                }
            }
        }
    }).to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn reserved_config_is_not_public() {
        let schema = super::parameters_schema();
        assert!(!schema.contains("__config"));
    }

    #[test]
    fn source_is_published_as_an_object_for_local_tool_grammars() {
        let schema: serde_json::Value =
            serde_json::from_str(&super::parameters_schema()).unwrap_or_else(|_| unreachable!());
        assert_eq!(
            schema.pointer("/properties/source/type"),
            Some(&serde_json::json!("object"))
        );
        assert!(schema.pointer("/properties/source/oneOf").is_none());
    }
}
