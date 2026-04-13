use super::*;

impl DynamicDeviceConfig {
    pub fn parse_device_name(&self, device_type: &str) -> Vec<String> {
        for device in self.device_cache.values() {
            if device_type.contains(&device.display_name) || device_type.contains(&device.id) {
                return self.extract_name_parts(&device.display_name);
            }
        }

        self.basic_name_parsing(device_type)
    }

    pub(super) fn extract_name_parts(&self, display_name: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut in_parentheses = false;
        let mut current_word = String::new();

        for ch in display_name.chars() {
            match ch {
                '(' => {
                    if !current_word.is_empty() {
                        parts.push(current_word.clone());
                        current_word.clear();
                    }
                    in_parentheses = true;
                }
                ')' => {
                    in_parentheses = false;
                }
                ' ' if !in_parentheses => {
                    if !current_word.is_empty() {
                        parts.push(current_word.clone());
                        current_word.clear();
                    }
                }
                _ if !in_parentheses => {
                    current_word.push(ch);
                }
                _ => {}
            }
        }

        if !current_word.is_empty() {
            parts.push(current_word);
        }

        parts
            .into_iter()
            .take(MAX_DEVICE_NAME_PARTS_DISPLAY)
            .collect()
    }

    fn basic_name_parsing(&self, device_type: &str) -> Vec<String> {
        self.extract_name_parts(device_type)
    }
}
