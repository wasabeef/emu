use super::{ABI_REGEX, AVD_NAME_REGEX, DEVICE_REGEX, PATH_REGEX, TARGET_REGEX};

/// AVD list parser for better testability.
pub(super) struct AvdListParser<'a> {
    lines: std::str::Lines<'a>,
    pub(super) current_device_info: Option<(String, String, String, String, String)>,
    pub(super) current_target_full: String,
}

impl<'a> AvdListParser<'a> {
    pub(super) fn new(output: &'a str) -> Self {
        Self {
            lines: output.lines(),
            current_device_info: None,
            current_target_full: String::new(),
        }
    }

    pub(super) fn parse_next_device(&mut self) -> Option<(String, String, String, String, String)> {
        for line in self.lines.by_ref() {
            let trimmed_line = line.trim();

            if self.current_device_info.is_some() && line.starts_with("          Based on:") {
                self.current_target_full.push(' ');
                self.current_target_full.push_str(trimmed_line);
            }

            if trimmed_line.starts_with("---") || trimmed_line.is_empty() {
                if let Some((name, path, mut target, abi, device)) = self.current_device_info.take()
                {
                    if !self.current_target_full.is_empty() {
                        target.push_str(&self.current_target_full);
                        self.current_target_full.clear();
                    }
                    return Some((name, path, target, abi, device));
                }
                continue;
            }

            if let Some(captures) = AVD_NAME_REGEX.captures(trimmed_line) {
                if let Some(name) = captures.get(1) {
                    self.current_device_info = Some((
                        name.as_str().to_string(),
                        String::new(),
                        String::new(),
                        String::new(),
                        String::new(),
                    ));
                }
            } else if let Some(captures) = PATH_REGEX.captures(trimmed_line) {
                if let Some(path) = captures.get(1) {
                    if let Some(ref mut info) = self.current_device_info {
                        info.1 = path.as_str().to_string();
                    }
                }
            } else if let Some(captures) = TARGET_REGEX.captures(trimmed_line) {
                if let Some(target) = captures.get(1) {
                    if let Some(ref mut info) = self.current_device_info {
                        info.2 = target.as_str().to_string();
                    }
                }
            } else if let Some(captures) = ABI_REGEX.captures(trimmed_line) {
                if let Some(abi) = captures.get(1) {
                    if let Some(ref mut info) = self.current_device_info {
                        info.3 = abi.as_str().to_string();
                    }
                }
            } else if let Some(captures) = DEVICE_REGEX.captures(trimmed_line) {
                if let Some(device) = captures.get(1) {
                    if let Some(ref mut info) = self.current_device_info {
                        info.4 = device.as_str().to_string();
                    }
                }
            }
        }

        if let Some((name, path, mut target, abi, device)) = self.current_device_info.take() {
            if !self.current_target_full.is_empty() {
                target.push_str(&self.current_target_full);
                self.current_target_full.clear();
            }
            return Some((name, path, target, abi, device));
        }

        None
    }
}
