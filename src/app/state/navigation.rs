use super::{AppState, Panel};

impl AppState {
    /// Switches between Android and iOS panels.
    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Android => Panel::Ios,
            Panel::Ios => Panel::Android,
        };
    }

    /// Moves selection up in the current device list.
    /// Wraps around from top to bottom when reaching the first item.
    pub fn move_up(&mut self) {
        match self.active_panel {
            Panel::Android => {
                if !self.android_devices.is_empty() {
                    if self.selected_android > 0 {
                        self.selected_android -= 1;
                    } else {
                        self.selected_android = self.android_devices.len() - 1;
                    }
                    self.update_android_scroll_offset();
                }
            }
            Panel::Ios => {
                if !self.ios_devices.is_empty() {
                    if self.selected_ios > 0 {
                        self.selected_ios -= 1;
                    } else {
                        self.selected_ios = self.ios_devices.len() - 1;
                    }
                    self.update_ios_scroll_offset();
                }
            }
        }
    }

    /// Moves selection down in the current device list.
    /// Wraps around from bottom to top when reaching the last item.
    pub fn move_down(&mut self) {
        match self.active_panel {
            Panel::Android => {
                if !self.android_devices.is_empty() {
                    if self.selected_android < self.android_devices.len() - 1 {
                        self.selected_android += 1;
                    } else {
                        self.selected_android = 0;
                    }
                    self.update_android_scroll_offset();
                }
            }
            Panel::Ios => {
                if !self.ios_devices.is_empty() {
                    if self.selected_ios < self.ios_devices.len() - 1 {
                        self.selected_ios += 1;
                    } else {
                        self.selected_ios = 0;
                    }
                    self.update_ios_scroll_offset();
                }
            }
        }
    }

    /// Moves device selection by a specified number of steps.
    /// Positive steps move down/right, negative steps move up/left.
    /// Handles wrapping at list boundaries.
    pub fn move_by_steps(&mut self, steps: i32) {
        if steps == 0 {
            return;
        }

        match self.active_panel {
            Panel::Android => {
                let device_count = self.android_devices.len();
                if device_count == 0 {
                    return;
                }

                let current = self.selected_android as i32;
                let new_pos = if steps > 0 {
                    let raw_pos = current + steps;
                    (raw_pos % device_count as i32) as usize
                } else {
                    let raw_pos = current + steps;
                    if raw_pos < 0 {
                        let wrapped = device_count as i32 + (raw_pos % device_count as i32);
                        (wrapped % device_count as i32) as usize
                    } else {
                        raw_pos as usize
                    }
                };

                self.selected_android = new_pos;
                self.update_android_scroll_offset();
            }
            Panel::Ios => {
                let device_count = self.ios_devices.len();
                if device_count == 0 {
                    return;
                }

                let current = self.selected_ios as i32;
                let new_pos = if steps > 0 {
                    let raw_pos = current + steps;
                    (raw_pos % device_count as i32) as usize
                } else {
                    let raw_pos = current + steps;
                    if raw_pos < 0 {
                        let wrapped = device_count as i32 + (raw_pos % device_count as i32);
                        (wrapped % device_count as i32) as usize
                    } else {
                        raw_pos as usize
                    }
                };

                self.selected_ios = new_pos;
                self.update_ios_scroll_offset();
            }
        }
    }

    /// Helper method to update Android scroll offset.
    /// Currently empty as scroll offset is calculated dynamically during rendering.
    fn update_android_scroll_offset(&mut self) {
        // No need to update here - render function will calculate dynamically
    }

    /// Helper method to update iOS scroll offset.
    /// Currently empty as scroll offset is calculated dynamically during rendering.
    fn update_ios_scroll_offset(&mut self) {
        // No need to update here - render function will calculate dynamically
    }

    /// Calculates the appropriate scroll offset for the Android device list.
    /// Ensures the selected item is visible within the available height.
    pub fn get_android_scroll_offset(&self, available_height: usize) -> usize {
        if self.android_devices.len() <= available_height || available_height == 0 {
            return 0;
        }

        let selected = self.selected_android;
        let current_offset = self.android_scroll_offset;

        if selected < current_offset {
            selected
        } else if selected >= current_offset + available_height {
            selected.saturating_sub(available_height.saturating_sub(1))
        } else {
            current_offset
        }
    }

    /// Calculates the appropriate scroll offset for the iOS device list.
    /// Ensures the selected item is visible within the available height.
    pub fn get_ios_scroll_offset(&self, available_height: usize) -> usize {
        if self.ios_devices.len() <= available_height || available_height == 0 {
            return 0;
        }

        let selected = self.selected_ios;
        let current_offset = self.ios_scroll_offset;

        if selected < current_offset {
            selected
        } else if selected >= current_offset + available_height {
            selected.saturating_sub(available_height.saturating_sub(1))
        } else {
            current_offset
        }
    }
}
