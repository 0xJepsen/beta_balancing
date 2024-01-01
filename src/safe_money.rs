pub trait Currency {
    fn name() -> &'static str;
    fn symbol() -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct USD {
    pub name: &'static str,
    pub symbol: &'static str,
    pub amount: f64,
}

impl USD {
    pub fn new(amount: f64) -> Self {
        Self {
            name: "United States Dollar",
            symbol: "USD",
            amount,
        }
    }
}

impl std::ops::SubAssign<f64> for USD {
    fn sub_assign(&mut self, other: f64) {
        self.amount -= other;
    }
}
impl std::ops::AddAssign<f64> for USD {
    fn add_assign(&mut self, other: f64) {
        self.amount += other;
    }
}

impl std::ops::AddAssign<USD> for USD {
    fn add_assign(&mut self, other: USD) {
        self.amount += other.amount;
    }
}

impl std::ops::SubAssign<USD> for USD {
    fn sub_assign(&mut self, other: USD) {
        self.amount -= other.amount;
    }
}

impl std::convert::From<f64> for USD {
    fn from(amount: f64) -> Self {
        Self {
            name: "United States Dollar",
            symbol: "USD",
            amount,
        }
    }
}

impl Add<USD> for f64 {
    type Output = USD;

    fn add(self, other: USD) -> USD {
        USD {
            amount: self + other.amount,
            ..other
        }
    }
}

impl Div<USD> for f64 {
    type Output = f64;

    fn div(self, other: USD) -> f64 {
        self / other.amount
    }
}

impl Mul<USD> for f64 {
    type Output = USD;

    fn mul(self, other: USD) -> USD {
        USD {
            amount: self * other.amount,
            ..other
        }
    }
}
impl std::ops::Neg for USD {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            amount: -self.amount,
            ..self
        }
    }
}

impl USD {
    pub fn abs(&self) -> Self {
        Self {
            amount: self.amount.abs(),
            ..*self
        }
    }
}

impl Sub for USD {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            amount: self.amount - other.amount,
            ..self
        }
    }
}
impl Add for USD {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            amount: self.amount + other.amount,
            ..self
        }
    }
}

impl Mul for USD {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            amount: self.amount * other.amount,
            ..self
        }
    }
}

impl Div for USD {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            amount: self.amount / other.amount,
            ..self
        }
    }
}

impl std::fmt::Display for USD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} {}", self.amount, self.symbol)
    }
}

impl Currency for USD {
    fn name() -> &'static str {
        "United States Dollar"
    }

    fn symbol() -> &'static str {
        "USD"
    }
}

pub struct Dense<C: Currency> {
    amount: f64,
    _currency: PhantomData<C>,
}

impl<C: Currency> Add for Dense<C> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            amount: self.amount + other.amount,
            _currency: PhantomData,
        }
    }
}
impl<C: Currency> Sub for Dense<C> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            amount: self.amount - other.amount,
            _currency: PhantomData,
        }
    }
}

impl<C: Currency> Mul for Dense<C> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            amount: self.amount * other.amount,
            _currency: PhantomData,
        }
    }
}

impl<C: Currency> Div for Dense<C> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            amount: self.amount / other.amount,
            _currency: PhantomData,
        }
    }
}

impl<C: Currency> Add for Discrete<C> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            cents: self.cents + other.cents,
            _currency: PhantomData,
        }
    }
}

impl<C: Currency> Sub for Discrete<C> {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            cents: self.cents - other.cents,
            _currency: PhantomData,
        }
    }
}

impl<C: Currency> Mul for Discrete<C> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self {
            cents: self.cents * other.cents,
            _currency: PhantomData,
        }
    }
}

impl<C: Currency> Div for Discrete<C> {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        Self {
            cents: self.cents / other.cents,
            _currency: PhantomData,
        }
    }
}

use std::{
    fmt,
    marker::PhantomData,
    ops::{Add, Div, Mul, Sub},
};

impl<C: Currency> fmt::Display for Dense<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, C::symbol())
    }
}

impl<C: Currency> fmt::Debug for Dense<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Dense {{ amount: {}, currency: {} }}",
            self.amount,
            C::symbol()
        )
    }
}

impl<C: Currency> fmt::Display for Discrete<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} cents in {}", self.cents, C::symbol())
    }
}

impl<C: Currency> fmt::Debug for Discrete<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Discrete {{ cents: {}, currency: {} }}",
            self.cents,
            C::symbol()
        )
    }
}

impl<C: Currency> From<f64> for Dense<C> {
    fn from(amount: f64) -> Self {
        Self {
            amount,
            _currency: PhantomData,
        }
    }
}

pub struct Discrete<C: Currency> {
    cents: i64,
    _currency: PhantomData<C>,
}

impl<C: Currency> From<f64> for Discrete<C> {
    fn from(amount: f64) -> Self {
        Self {
            cents: (amount * 100.0) as i64, // Convert dollars to cents
            _currency: PhantomData,
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_dense_add() {
        let money1: Dense<USD> = Dense::from(10.0);
        let money2: Dense<USD> = Dense::from(20.0);
        let result = money1 + money2;
        assert_eq!(result.amount, 30.0);
    }

    #[test]
    fn test_discrete_add() {
        let money1: Discrete<USD> = Discrete::from(10.0); // 1000 cents = 10 USD
        let money2: Discrete<USD> = Discrete::from(20.0); // 2000 cents = 20 USD
        let result = money1 + money2;
        assert_eq!(result.cents, 3000); // 3000 cents = 30 USD
    }

    #[test]
    fn test_dense_sub() {
        let money1: Dense<USD> = Dense::from(10.0);
        let money2: Dense<USD> = Dense::from(20.0);
        let result = money1 - money2;
        assert_eq!(result.amount, -10.0);
    }

    #[test]
    fn test_discrete_sub() {
        let money1: Discrete<USD> = Discrete::from(10.0); // 1000 cents = 10 USD
        let money2: Discrete<USD> = Discrete::from(20.0); // 2000 cents = 20 USD
        let result = money1 - money2;
        assert_eq!(result.cents, -1000); // -1000 cents = -10 USD
    }

    #[test]
    fn test_dense_mul() {
        let money1: Dense<USD> = Dense::from(10.0);
        let money2: Dense<USD> = Dense::from(20.0);
        let result = money1 * money2;
        assert_eq!(result.amount, 200.0);
    }

    #[test]
    fn test_discrete_mul() {
        let money1: Discrete<USD> = Discrete::from(10.0); // 1000 cents = 10 USD
        let money2: Discrete<USD> = Discrete::from(20.0); // 2000 cents = 20 USD
        let result = money1 * money2;
        assert_eq!(result.cents, 2000000); // 2000000 cents = 20000 USD
    }

    #[test]
    fn test_dense_div() {
        let money1: Dense<USD> = Dense::from(10.0);
        let money2: Dense<USD> = Dense::from(20.0);
        let result = money1 / money2;
        assert_eq!(result.amount, 0.5);
    }

    #[test]
    fn test_discrete_div() {
        let money1: Discrete<USD> = Discrete::from(10.0); // 1000 cents = 10 USD
        let money2: Discrete<USD> = Discrete::from(20.0); // 2000 cents = 20 USD
        let result = money1 / money2;
        assert_eq!(result.cents, 0); // 0 cents = 0 USD
    }

    #[test]
    fn test_dense_display() {
        let money: Dense<USD> = Dense::from(10.0);
        assert_eq!(format!("{}", money), "10 USD");
    }

    #[test]
    fn test_discrete_display() {
        let money: Discrete<USD> = Discrete::from(10.0); // 1000 cents = 10 USD
        assert_eq!(format!("{}", money), "1000 cents in USD");
    }
}
