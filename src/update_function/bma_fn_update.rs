use crate::update_function::BmaUpdateFunction;
use crate::update_function::expression_token::BmaExpressionToken;
use crate::update_function::parser::{parse_bma_fn_tokens, parse_bma_formula};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for BmaUpdateFunction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for BmaUpdateFunction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match BmaUpdateFunction::parse_from_str(&value, &[]) {
            Ok(tree) => Ok(tree),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}

impl BmaUpdateFunction {
    /// "Parse" new [BmaUpdateFunction] tree from a list of [BmaExpressionToken] objects.
    pub fn from_tokens(tokens: &[BmaExpressionToken]) -> Result<BmaUpdateFunction, String> {
        parse_bma_fn_tokens(tokens)
    }

    /// Parse new [BmaUpdateFunction] tree directly from a string representation.
    ///
    /// Arg `variables` is a map of variable IDs to their names. It is needed because there are
    /// some weird format differences between different variants, and a variable can be referenced
    /// by either its ID or its name. We convert everything to IDs for easier processing.
    pub fn parse_from_str(
        function_str: &str,
        variables: &[(u32, String)],
    ) -> Result<BmaUpdateFunction, String> {
        parse_bma_formula(function_str, variables)
    }
}
