use rug::Integer;
use std::io::{self, Read, Write};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Qp {
    pub numerator: Integer,
    pub valuation: i16,
}

#[derive(Clone, Copy, Debug)]
pub struct QpOp {
    prime: i32,
}

impl QpOp {
    pub const fn new(prime: i32) -> Self {
        Self { prime }
    }

    pub const fn prime(&self) -> i32 {
        self.prime
    }

    pub fn power_p(&self, i: u32) -> Integer {
        let mut res = Integer::from(1);
        for _ in 0..i {
            res *= self.prime;
        }
        res
    }

    pub fn power_p_int(&self, i: u32) -> i32 {
        let mut res = 1;
        for _ in 0..i {
            res *= self.prime;
        }
        res
    }

    pub fn simplify(&self, x: &mut Qp) {
        if x.numerator.is_zero() {
            x.valuation = 0;
            return;
        }

        let p = Integer::from(self.prime);
        while (&x.numerator % &p).is_zero() {
            x.numerator /= &p;
            x.valuation += 1;
        }
    }

    pub fn output(&self, x: &Qp) -> String {
        format!("{}({})", x.numerator, x.valuation)
    }

    pub fn save(&self, x: &Qp, writer: &mut impl Write) -> io::Result<()> {
        let numerator_bytes = self.save_to_vec(&x.numerator);

        writer.write_all(&x.valuation.to_le_bytes())?;

        let len = i16::try_from(numerator_bytes.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "numerator too long"))?;
        writer.write_all(&len.to_le_bytes())?;
        writer.write_all(&numerator_bytes)?;
        Ok(())
    }

    pub fn load(&self, reader: &mut impl Read) -> io::Result<Qp> {
        let mut valuation_bytes = [0u8; 2];
        reader.read_exact(&mut valuation_bytes)?;
        let valuation = i16::from_le_bytes(valuation_bytes);

        let mut len_bytes = [0u8; 2];
        reader.read_exact(&mut len_bytes)?;
        let len = i16::from_le_bytes(len_bytes);
        let len = usize::try_from(len)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid numerator length"))?;

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;

        Ok(Qp {
            numerator: self.load_from_vec(&buf),
            valuation,
        })
    }

    pub fn add(&self, x: &Qp, y: &Qp) -> Qp {
        if x.valuation < y.valuation {
            return Qp {
                numerator: &x.numerator
                    + &y.numerator * self.power_p((y.valuation - x.valuation) as u32),
                valuation: x.valuation,
            };
        }

        if x.valuation > y.valuation {
            return Qp {
                numerator: &y.numerator
                    + &x.numerator * self.power_p((x.valuation - y.valuation) as u32),
                valuation: y.valuation,
            };
        }

        let mut z = Qp {
            numerator: &x.numerator + &y.numerator,
            valuation: x.valuation,
        };
        self.simplify(&mut z);
        z
    }

    pub fn zero(&self) -> Qp {
        Qp {
            numerator: Integer::new(),
            valuation: 0,
        }
    }

    pub fn is_zero(&self, x: &Qp) -> bool {
        x.numerator.is_zero()
    }

    pub fn minus(&self, x: &Qp) -> Qp {
        Qp {
            numerator: -&x.numerator,
            valuation: x.valuation,
        }
    }

    pub fn multiply(&self, x: &Qp, y: &Qp) -> Qp {
        Qp {
            numerator: &x.numerator * &y.numerator,
            valuation: x.valuation + y.valuation,
        }
    }

    pub fn unit(&self, x: i32) -> Qp {
        let mut result = Qp {
            numerator: Integer::from(x),
            valuation: 0,
        };
        self.simplify(&mut result);
        result
    }

    pub fn invertible(&self, x: &Qp) -> bool {
        !self.is_zero(x)
    }

    pub fn inverse(&self, _x: &Qp) -> Qp {
        panic!("not implemented");
    }

    pub fn construct(&self, num: i32, val: i16) -> Qp {
        Qp {
            numerator: Integer::from(num),
            valuation: val,
        }
    }

    pub fn int_part(&self, x: &Qp) -> u64 {
        if x.valuation < 0 {
            panic!("not integral: {}", self.output(x));
        }

        let mut ip = x.numerator.clone() * self.power_p(x.valuation as u32);
        let t64 = Integer::from(self.prime).pow(40u32);
        ip %= &t64;
        if ip < 0 {
            ip += t64;
        }
        ip.to_u64_wrapping()
    }

    pub fn save_as_integer(&self, x: &Qp, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&self.int_part(x).to_le_bytes())
    }

    pub fn output_integer(&self, x: &Qp) -> String {
        self.int_part(x).to_string()
    }

    fn save_to_vec(&self, x: &Integer) -> Vec<u8> {
        if x.is_zero() {
            return vec![0];
        }

        let mut result = Vec::new();
        result.push(if x > &Integer::ZERO { 1 } else { 2 });

        let mut n = x.abs();
        while !n.is_zero() {
            let digit = (&n % 256u32).to_u32_wrapping() as u8;
            result.push(digit);
            n /= 256u32;
        }

        result
    }

    fn load_from_vec(&self, buf: &[u8]) -> Integer {
        if buf.is_empty() {
            return Integer::new();
        }

        let mut res = Integer::new();
        let mut vl = Integer::from(1);
        for &b in buf.iter().skip(1) {
            res += &vl * Integer::from(b);
            vl *= 256u32;
        }

        if buf[0] == 2 {
            res = -res;
        }
        res
    }
}

pub const Q3_OP: QpOp = QpOp { prime: 3 };
