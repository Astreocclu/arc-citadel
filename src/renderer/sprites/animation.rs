//! Animation state machine for sprites.

/// Animation states for entities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AnimationState {
    #[default]
    Idle,
    Move,
    Attack,
    Hit,
    Die,
    Rout,
}

/// Controls animation playback for a single entity.
#[derive(Clone, Debug)]
pub struct AnimationController {
    /// Current animation state.
    pub current_state: AnimationState,
    /// Current frame within the animation.
    pub current_frame: u8,
    /// Time accumulator for frame advancement.
    pub frame_timer: f32,
    /// Facing direction (0-7 for 8 directions, 0 = East, counter-clockwise).
    pub direction: u8,
    /// Whether the current animation has finished (for non-looping).
    pub finished: bool,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationController {
    /// Create a new animation controller in idle state.
    pub fn new() -> Self {
        Self {
            current_state: AnimationState::Idle,
            current_frame: 0,
            frame_timer: 0.0,
            direction: 0,
            finished: false,
        }
    }

    /// Update animation, returns true if frame changed.
    pub fn update(&mut self, dt: f32, animation_data: &AnimationData) -> bool {
        if self.finished {
            return false;
        }

        self.frame_timer += dt;

        let frame_duration = animation_data.frame_duration(self.current_state);
        let frame_count = animation_data.frame_count(self.current_state);

        if frame_count == 0 {
            return false;
        }

        if self.frame_timer >= frame_duration {
            self.frame_timer -= frame_duration;

            let next_frame = self.current_frame + 1;
            if next_frame >= frame_count {
                if animation_data.is_looping(self.current_state) {
                    self.current_frame = 0;
                } else {
                    self.current_frame = frame_count - 1;
                    self.finished = true;
                }
            } else {
                self.current_frame = next_frame;
            }
            return true;
        }
        false
    }

    /// Set the animation state. Resets frame if state changed.
    pub fn set_state(&mut self, state: AnimationState) {
        if self.current_state != state {
            self.current_state = state;
            self.current_frame = 0;
            self.frame_timer = 0.0;
            self.finished = false;
        }
    }

    /// Set facing direction from an angle in radians.
    /// 0 = East, counter-clockwise: 1=NE, 2=N, 3=NW, 4=W, 5=SW, 6=S, 7=SE.
    pub fn set_direction_from_angle(&mut self, angle: f32) {
        // Normalize angle to 0-1 range (handles negative angles)
        let normalized = (angle / std::f32::consts::TAU).rem_euclid(1.0);
        self.direction = ((normalized * 8.0).round() as u8) % 8;
    }

    /// Get the frame index in the sprite atlas.
    pub fn atlas_frame(&self, animation_data: &AnimationData) -> u32 {
        let base = animation_data.base_frame(self.current_state);
        let direction_offset = self.direction as u32 * animation_data.frames_per_direction() as u32;
        base + direction_offset + self.current_frame as u32
    }

    /// Get UV coordinates for the current frame.
    pub fn uv_coords(&self, animation_data: &AnimationData) -> ([f32; 2], [f32; 2]) {
        let frame = self.atlas_frame(animation_data);
        animation_data.frame_uv(frame)
    }
}

/// Animation data for a sprite type (e.g., "human_soldier").
#[derive(Clone, Debug)]
pub struct AnimationData {
    /// Frames per animation state.
    pub idle_frames: u8,
    pub move_frames: u8,
    pub attack_frames: u8,
    pub hit_frames: u8,
    pub die_frames: u8,
    pub rout_frames: u8,

    /// Number of directions (typically 1, 4, or 8).
    pub directions: u8,

    /// Seconds per frame (default animation speed).
    pub frame_duration: f32,

    /// Atlas layout.
    pub atlas_columns: u32,
    pub atlas_rows: u32,
    pub sprite_width: u32,
    pub sprite_height: u32,
    pub atlas_width: u32,
    pub atlas_height: u32,
}

impl Default for AnimationData {
    fn default() -> Self {
        Self {
            idle_frames: 4,
            move_frames: 6,
            attack_frames: 4,
            hit_frames: 2,
            die_frames: 4,
            rout_frames: 4,
            directions: 8,
            frame_duration: 0.1,
            atlas_columns: 8,
            atlas_rows: 8,
            sprite_width: 32,
            sprite_height: 32,
            atlas_width: 256,
            atlas_height: 256,
        }
    }
}

impl AnimationData {
    /// Get frame count for a state.
    pub fn frame_count(&self, state: AnimationState) -> u8 {
        match state {
            AnimationState::Idle => self.idle_frames,
            AnimationState::Move => self.move_frames,
            AnimationState::Attack => self.attack_frames,
            AnimationState::Hit => self.hit_frames,
            AnimationState::Die => self.die_frames,
            AnimationState::Rout => self.rout_frames,
        }
    }

    /// Get frame duration for a state (could vary per state).
    pub fn frame_duration(&self, _state: AnimationState) -> f32 {
        self.frame_duration
    }

    /// Whether the animation loops.
    pub fn is_looping(&self, state: AnimationState) -> bool {
        match state {
            AnimationState::Idle | AnimationState::Move | AnimationState::Rout => true,
            AnimationState::Attack | AnimationState::Hit | AnimationState::Die => false,
        }
    }

    /// Get base frame index for a state.
    pub fn base_frame(&self, state: AnimationState) -> u32 {
        let mut offset = 0u32;
        let frames_per_dir = self.frames_per_direction() as u32;

        match state {
            AnimationState::Idle => offset,
            AnimationState::Move => {
                offset += self.idle_frames as u32 * frames_per_dir;
                offset
            }
            AnimationState::Attack => {
                offset += self.idle_frames as u32 * frames_per_dir;
                offset += self.move_frames as u32 * frames_per_dir;
                offset
            }
            AnimationState::Hit => {
                offset += self.idle_frames as u32 * frames_per_dir;
                offset += self.move_frames as u32 * frames_per_dir;
                offset += self.attack_frames as u32 * frames_per_dir;
                offset
            }
            AnimationState::Die => {
                offset += self.idle_frames as u32 * frames_per_dir;
                offset += self.move_frames as u32 * frames_per_dir;
                offset += self.attack_frames as u32 * frames_per_dir;
                offset += self.hit_frames as u32 * frames_per_dir;
                offset
            }
            AnimationState::Rout => {
                offset += self.idle_frames as u32 * frames_per_dir;
                offset += self.move_frames as u32 * frames_per_dir;
                offset += self.attack_frames as u32 * frames_per_dir;
                offset += self.hit_frames as u32 * frames_per_dir;
                offset += self.die_frames as u32 * frames_per_dir;
                offset
            }
        }
    }

    /// Frames per direction (1 if no directional sprites).
    pub fn frames_per_direction(&self) -> u8 {
        if self.directions > 1 {
            1
        } else {
            1
        }
    }

    /// Get UV coordinates for a frame index.
    pub fn frame_uv(&self, frame: u32) -> ([f32; 2], [f32; 2]) {
        let col = frame % self.atlas_columns;
        let row = frame / self.atlas_columns;

        let u0 = (col * self.sprite_width) as f32 / self.atlas_width as f32;
        let v0 = (row * self.sprite_height) as f32 / self.atlas_height as f32;
        let u1 = ((col + 1) * self.sprite_width) as f32 / self.atlas_width as f32;
        let v1 = ((row + 1) * self.sprite_height) as f32 / self.atlas_height as f32;

        ([u0, v0], [u1 - u0, v1 - v0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_state_transition() {
        let mut controller = AnimationController::new();
        assert_eq!(controller.current_state, AnimationState::Idle);

        controller.set_state(AnimationState::Move);
        assert_eq!(controller.current_state, AnimationState::Move);
        assert_eq!(controller.current_frame, 0);
    }

    #[test]
    fn test_animation_frame_advance() {
        let mut controller = AnimationController::new();
        let data = AnimationData::default();

        // Advance time less than frame duration
        assert!(!controller.update(0.05, &data));
        assert_eq!(controller.current_frame, 0);

        // Advance past frame duration
        assert!(controller.update(0.06, &data));
        assert_eq!(controller.current_frame, 1);
    }

    #[test]
    fn test_animation_loop() {
        let mut controller = AnimationController::new();
        let data = AnimationData {
            idle_frames: 2,
            ..Default::default()
        };

        // Advance to last frame
        controller.update(0.15, &data);
        assert_eq!(controller.current_frame, 1);

        // Loop back to first frame
        controller.update(0.1, &data);
        assert_eq!(controller.current_frame, 0);
        assert!(!controller.finished);
    }

    #[test]
    fn test_non_looping_animation() {
        let mut controller = AnimationController::new();
        controller.set_state(AnimationState::Die);

        let data = AnimationData {
            die_frames: 2,
            ..Default::default()
        };

        // Advance to last frame
        controller.update(0.1, &data);
        controller.update(0.1, &data);

        assert!(controller.finished);
        assert_eq!(controller.current_frame, 1);

        // Should stay on last frame
        controller.update(0.1, &data);
        assert_eq!(controller.current_frame, 1);
    }

    #[test]
    fn test_direction_from_angle() {
        let mut controller = AnimationController::new();

        controller.set_direction_from_angle(0.0); // East
        assert_eq!(controller.direction, 0);

        controller.set_direction_from_angle(std::f32::consts::FRAC_PI_2); // North
        assert_eq!(controller.direction, 2);

        controller.set_direction_from_angle(std::f32::consts::PI); // West
        assert_eq!(controller.direction, 4);

        controller.set_direction_from_angle(-std::f32::consts::FRAC_PI_2); // South
        assert_eq!(controller.direction, 6);
    }

    #[test]
    fn test_frame_uv() {
        let data = AnimationData {
            atlas_columns: 4,
            atlas_rows: 4,
            sprite_width: 32,
            sprite_height: 32,
            atlas_width: 128,
            atlas_height: 128,
            ..Default::default()
        };

        let (offset, size) = data.frame_uv(0);
        assert_eq!(offset, [0.0, 0.0]);
        assert_eq!(size, [0.25, 0.25]);

        let (offset, size) = data.frame_uv(5);
        assert_eq!(offset, [0.25, 0.25]); // col 1, row 1
        assert_eq!(size, [0.25, 0.25]);
    }
}
