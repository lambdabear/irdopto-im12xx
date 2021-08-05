#![no_std]

use fixedvec::FixedVec;
use rmodbus::{client::ModbusRequest, ErrorKind, ModbusProto};

pub const BAUT: u32 = 4800;

const MODBUS_ADDR: u8 = 1;
const REG_ADDR: u16 = 0x48;
const REG_COUNT: u16 = 8;

#[derive(Debug)]
pub enum Error {
    ModbusErr(ErrorKind),
    NoRequest,
    ResponseParseErr,
}

pub struct Im12xx<'a> {
    req_bytes: FixedVec<'a, u8>,
    req: ModbusRequest,
}

impl<'a> Im12xx<'a> {
    pub fn new(buffer: &'a mut [u8; 256]) -> Result<Self, Error> {
        let mut modbus_req = ModbusRequest::new(MODBUS_ADDR, ModbusProto::Rtu);
        let mut request = FixedVec::new(&mut buffer[..]);
        modbus_req
            .generate_get_holdings(REG_ADDR, REG_COUNT, &mut request)
            .map_err(|e| Error::ModbusErr(e))?;

        Ok(Self {
            req_bytes: request,
            req: modbus_req,
        })
    }

    pub fn request(&mut self) -> &[u8] {
        self.req_bytes.as_slice()
    }

    pub fn response(&mut self, data: &[u8]) -> Result<PowerState, Error> {
        let len = data.len();
        if len != 37 {
            return Err(Error::ResponseParseErr);
        }

        self.req.parse_ok(data).map_err(|e| Error::ModbusErr(e))?;
        PowerState::from(&data[3..len - 2])
    }
}

#[derive(Debug, PartialEq)]
pub struct PowerState {
    pub voltage: f32,
    pub current: f32,
    pub active_power: f32,
    pub active_energy: f32,
    pub power_factor: f32,
    pub co2_emissions: f32,
    pub temperature: f32,
    pub frequency: f32,
}

impl PowerState {
    pub fn from(buf: &[u8]) -> Result<Self, Error> {
        if buf.len() != 32 {
            return Err(Error::ResponseParseErr);
        }

        let mut data = [0; 8];
        for i in 0..8 {
            data[i] =
                u32::from_be_bytes([buf[i * 4], buf[i * 4 + 1], buf[i * 4 + 2], buf[i * 4 + 3]]);
        }

        Ok(PowerState {
            voltage: data[0] as f32 * 0.0001,
            current: data[1] as f32 * 0.0001,
            active_power: data[2] as f32 * 0.0001,
            active_energy: data[3] as f32 * 0.0001,
            power_factor: data[4] as f32 * 0.001,
            co2_emissions: data[5] as f32 * 0.0001,
            temperature: data[6] as f32 * 0.01,
            frequency: data[7] as f32 * 0.01,
        })
    }

    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut bytes = [0; 32];
        let data = [
            self.voltage,
            self.current,
            self.active_power,
            self.active_energy,
            self.power_factor,
            self.co2_emissions,
            self.temperature,
            self.frequency,
        ];

        for i in 0..8 {
            let b = data[i].to_be_bytes();
            bytes[i * 4] = b[0];
            bytes[i * 4 + 1] = b[1];
            bytes[i * 4 + 2] = b[2];
            bytes[i * 4 + 3] = b[3];
        }

        bytes
    }

    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let mut data = [0.0; 8];

        for i in 0..8 {
            data[i] = f32::from_be_bytes([
                bytes[i * 4],
                bytes[i * 4 + 1],
                bytes[i * 4 + 2],
                bytes[i * 4 + 3],
            ]);
        }

        PowerState {
            voltage: data[0],
            current: data[1],
            active_power: data[2],
            active_energy: data[3],
            power_factor: data[4],
            co2_emissions: data[5],
            temperature: data[6],
            frequency: data[7],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Im12xx, PowerState};
    #[test]
    fn it_works() {
        let mut buffer = [0; 256];
        let mut im12xx = Im12xx::new(&mut buffer).unwrap();

        assert_eq!(im12xx.request(), [1, 3, 0, 72, 0, 8, 196, 26]);

        let response_data = [
            0x01, 0x03, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xE7, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x0B, 0x54, 0x00, 0x00, 0x13, 0x88, 0xD9, 0xBE,
        ];
        let power_state = im12xx.response(&response_data).unwrap();
        assert_eq!(
            power_state,
            PowerState {
                voltage: 0.0,
                current: 0.0,
                active_power: 0.0,
                active_energy: 0.0,
                power_factor: 0.9990001,
                co2_emissions: 0.0,
                temperature: 29.0,
                frequency: 50.0
            }
        );

        let bytes = power_state.to_be_bytes();
        assert_eq!(PowerState::from_be_bytes(bytes), power_state);
    }
}
