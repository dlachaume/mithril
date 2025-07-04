use anyhow::anyhow;
use slog::Logger;

use mithril_client::MithrilResult;

use crate::configuration::ConfigParameters;

/// Context for the command execution
pub struct CommandContext {
    config_parameters: ConfigParameters,
    unstable_enabled: bool,
    json: bool,
    logger: Logger,
}

impl CommandContext {
    /// Create a new command context
    pub fn new(
        config_parameters: ConfigParameters,
        unstable_enabled: bool,
        json: bool,
        logger: Logger,
    ) -> Self {
        Self {
            config_parameters,
            unstable_enabled,
            json,
            logger,
        }
    }

    /// Check if unstable commands are enabled
    pub fn is_unstable_enabled(&self) -> bool {
        self.unstable_enabled
    }

    /// Check if JSON output is enabled
    pub fn is_json_output_enabled(&self) -> bool {
        self.json
    }

    /// Ensure that unstable commands are enabled
    pub fn require_unstable(
        &self,
        sub_command: &str,
        command_example: Option<&str>,
    ) -> MithrilResult<()> {
        if self.is_unstable_enabled() {
            Ok(())
        } else {
            let example = command_example.map(|e| format!(" {e}")).unwrap_or_default();
            Err(anyhow!(
                "The \"{sub_command}\" subcommand is only accepted using the --unstable flag.\n\n\
                ie: \"mithril-client --unstable {sub_command}{example}\""
            ))
        }
    }

    /// Get a reference to the configured parameters
    pub fn config_parameters(&self) -> &ConfigParameters {
        &self.config_parameters
    }

    /// Get a mutable reference to the configured parameters
    pub fn config_parameters_mut(&mut self) -> &mut ConfigParameters {
        &mut self.config_parameters
    }

    /// Get the shared logger
    pub fn logger(&self) -> &Logger {
        &self.logger
    }
}

#[cfg(test)]
mod tests {
    use slog::o;
    use std::collections::HashMap;

    use crate::configuration::{ConfigError, ConfigSource};

    use super::*;

    #[test]
    fn require_unstable_return_ok_if_unstable_enabled() {
        let unstable_enabled = true;
        let context = CommandContext::new(
            ConfigParameters::default(),
            unstable_enabled,
            true,
            Logger::root(slog::Discard, o!()),
        );

        let result = context.require_unstable("test", None);
        assert!(result.is_ok(), "Expected Ok, got {result:?}");
    }

    #[test]
    fn require_unstable_return_err_if_unstable_disabled() {
        let unstable_enabled = false;
        let context = CommandContext::new(
            ConfigParameters::default(),
            unstable_enabled,
            true,
            Logger::root(slog::Discard, o!()),
        );

        let result = context.require_unstable("test", None);
        assert!(result.is_err(), "Expected Err, got {result:?}");
    }

    #[test]
    fn can_edit_config_parameters() {
        struct ParamSource {
            key: String,
            value: String,
        }
        impl ConfigSource for ParamSource {
            fn collect(&self) -> Result<HashMap<String, String>, ConfigError> {
                Ok(HashMap::from([(self.key.clone(), self.value.clone())]))
            }
        }

        let mut context = CommandContext::new(
            ConfigParameters::default(),
            false,
            true,
            Logger::root(slog::Discard, o!()),
        );

        assert_eq!(context.config_parameters_mut().get("key"), None,);

        context
            .config_parameters_mut()
            .add_source(&ParamSource {
                key: "key".to_string(),
                value: "value".to_string(),
            })
            .unwrap();

        assert_eq!(
            context.config_parameters_mut().get("key"),
            Some("value".to_string())
        );
    }
}
