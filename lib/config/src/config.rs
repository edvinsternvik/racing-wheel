#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Config {
    pub gain: f32,
    pub expo: f32,
    pub max_rotation: u16,
    pub spring_gain: f32,
    pub spring_coefficient: f32,
    pub spring_saturation: f32,
    pub spring_deadband: f32,
    pub motor_max: f32,
    pub motor_deadband: f32,
    pub motor_frequency_hz: u16,
    pub update_frequency_hz: u16,
}

impl Config {
    pub fn into_bytes(&self, id: u8) -> [u8; 39] {
        [
            id,
            f32::to_le_bytes(self.gain)[0],
            f32::to_le_bytes(self.gain)[1],
            f32::to_le_bytes(self.gain)[2],
            f32::to_le_bytes(self.gain)[3],
            f32::to_le_bytes(self.expo)[0],
            f32::to_le_bytes(self.expo)[1],
            f32::to_le_bytes(self.expo)[2],
            f32::to_le_bytes(self.expo)[3],
            u16::to_le_bytes(self.max_rotation)[0],
            u16::to_le_bytes(self.max_rotation)[1],
            f32::to_le_bytes(self.spring_gain)[0],
            f32::to_le_bytes(self.spring_gain)[1],
            f32::to_le_bytes(self.spring_gain)[2],
            f32::to_le_bytes(self.spring_gain)[3],
            f32::to_le_bytes(self.spring_coefficient)[0],
            f32::to_le_bytes(self.spring_coefficient)[1],
            f32::to_le_bytes(self.spring_coefficient)[2],
            f32::to_le_bytes(self.spring_coefficient)[3],
            f32::to_le_bytes(self.spring_saturation)[0],
            f32::to_le_bytes(self.spring_saturation)[1],
            f32::to_le_bytes(self.spring_saturation)[2],
            f32::to_le_bytes(self.spring_saturation)[3],
            f32::to_le_bytes(self.spring_deadband)[0],
            f32::to_le_bytes(self.spring_deadband)[1],
            f32::to_le_bytes(self.spring_deadband)[2],
            f32::to_le_bytes(self.spring_deadband)[3],
            f32::to_le_bytes(self.motor_max)[0],
            f32::to_le_bytes(self.motor_max)[1],
            f32::to_le_bytes(self.motor_max)[2],
            f32::to_le_bytes(self.motor_max)[3],
            f32::to_le_bytes(self.motor_deadband)[0],
            f32::to_le_bytes(self.motor_deadband)[1],
            f32::to_le_bytes(self.motor_deadband)[2],
            f32::to_le_bytes(self.motor_deadband)[3],
            u16::to_le_bytes(self.motor_frequency_hz)[0],
            u16::to_le_bytes(self.motor_frequency_hz)[1],
            u16::to_le_bytes(self.update_frequency_hz)[0],
            u16::to_le_bytes(self.update_frequency_hz)[1],
        ]
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        Some(Config {
            gain: f32::from_le_bytes([
                *bytes.get(0)?,
                *bytes.get(1)?,
                *bytes.get(2)?,
                *bytes.get(3)?,
            ]),
            expo: f32::from_le_bytes([
                *bytes.get(4)?,
                *bytes.get(5)?,
                *bytes.get(6)?,
                *bytes.get(7)?,
            ]),
            max_rotation: u16::from_le_bytes([*bytes.get(8)?, *bytes.get(9)?]),
            spring_gain: f32::from_le_bytes([
                *bytes.get(10)?,
                *bytes.get(11)?,
                *bytes.get(12)?,
                *bytes.get(13)?,
            ]),
            spring_coefficient: f32::from_le_bytes([
                *bytes.get(14)?,
                *bytes.get(15)?,
                *bytes.get(16)?,
                *bytes.get(17)?,
            ]),
            spring_saturation: f32::from_le_bytes([
                *bytes.get(18)?,
                *bytes.get(19)?,
                *bytes.get(20)?,
                *bytes.get(21)?,
            ]),
            spring_deadband: f32::from_le_bytes([
                *bytes.get(22)?,
                *bytes.get(23)?,
                *bytes.get(24)?,
                *bytes.get(25)?,
            ]),
            motor_max: f32::from_le_bytes([
                *bytes.get(26)?,
                *bytes.get(27)?,
                *bytes.get(28)?,
                *bytes.get(29)?,
            ]),
            motor_deadband: f32::from_le_bytes([
                *bytes.get(30)?,
                *bytes.get(31)?,
                *bytes.get(32)?,
                *bytes.get(33)?,
            ]),
            motor_frequency_hz: u16::from_le_bytes([*bytes.get(34)?, *bytes.get(35)?]),
            update_frequency_hz: u16::from_le_bytes([*bytes.get(36)?, *bytes.get(37)?]),
        })
    }
}
