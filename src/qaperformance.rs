use std::collections::HashMap;

use qifi_rs::account::Trade;
use serde::{Deserialize, Serialize};

use crate::market_preset::MarketPreset;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QATradePair {
    pub open_datetime: i64,
    pub close_datetime: i64,
    pub is_buy: bool,
    pub code: String,
    pub amount: f64,
    pub open_price: f64,
    pub close_price: f64,
    pub open_trade_id: String,
    pub close_trade_id: String,
    pub pnl_ratio: f64,
    pub pnl_money: f64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Temp {
    pub amount: f64,
    pub direction: String,
    pub offset: String,
    pub datetime: i64,
    pub code: String,
    pub price: f64,
    pub trade_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QAPerformance {
    pub market_set: MarketPreset,
    pub pair: Vec<QATradePair>,
    pub temp: HashMap<String, Vec<Temp>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QARiskMessage {}

impl QAPerformance {
    pub fn new() -> Self {
        let mut temp = HashMap::new();
        temp.insert("BUY".to_string(), vec![]);
        temp.insert("SELL".to_string(), vec![]);
        QAPerformance {
            market_set: MarketPreset::new(),
            pair: vec![],
            temp,
        }
    }
    pub fn insert_trade(&mut self, trade: Trade) {
        match trade.offset.as_str() {
            "OPEN" => {
                let direction = trade.direction.as_str();
                let u = self.temp.get_mut(direction).unwrap();
                u.push(Temp {
                    amount: trade.volume.clone(),
                    direction: trade.direction.clone(),
                    offset: "OPEN".to_string(),
                    datetime: trade.trade_date_time.clone(),
                    code: trade.instrument_id.clone(),
                    price: trade.price.clone(),
                    trade_id: trade.trade_id.clone(),
                });
            }
            "CLOSE" => {
                let (raw_direction, is_buy) = match trade.direction.as_str() {
                    "BUY" => ("SELL", false),
                    "SELL" => ("BUY", true),
                    _ => ("", false),
                };
                let u = self.temp.get_mut(raw_direction).unwrap();
                //println!("{:#?}", u);

                let mut codeset = self.market_set.get(trade.instrument_id.as_ref());

                let f = u.get_mut(0).unwrap();

                if trade.volume > f.amount {
                    // close> raw ==> 注销继续loop

                    let pnl_money = codeset.unit_table as f64
                        * (trade.price.clone() - f.price.clone())
                        * f.amount.clone();
                    let pnl_ratio =
                        pnl_money / (f.price.clone() * f.amount.clone() * codeset.calc_coeff());
                    self.pair.push(QATradePair {
                        open_datetime: f.datetime.clone(),
                        close_datetime: trade.trade_date_time.clone(),
                        is_buy,
                        code: f.code.clone(),
                        amount: f.amount.clone(),
                        open_price: f.price.clone(),
                        close_price: trade.price.clone(),
                        open_trade_id: f.trade_id.clone(),
                        close_trade_id: trade.trade_id.clone(),
                        pnl_ratio,
                        pnl_money,
                    });
                    let mut new_t = trade.clone();

                    new_t.volume -= f.amount;
                    u.remove(0);
                    self.insert_trade(new_t)
                } else if trade.volume < f.amount {
                    let pnl_money = codeset.unit_table as f64
                        * (trade.price.clone() - f.price.clone())
                        * trade.volume.clone();
                    let pnl_ratio =
                        pnl_money / (f.price.clone() * trade.volume.clone() * codeset.calc_coeff());
                    self.pair.push(QATradePair {
                        open_datetime: f.datetime.clone(),
                        close_datetime: trade.trade_date_time.clone(),
                        is_buy,
                        code: f.code.clone(),
                        amount: trade.volume.clone(),
                        open_price: f.price.clone(),
                        close_price: trade.price.clone(),
                        open_trade_id: f.trade_id.clone(),
                        close_trade_id: trade.trade_id.clone(),
                        pnl_ratio,
                        pnl_money,
                    });
                    f.amount -= trade.volume.clone();

                    //u.insert(0, f.clone());
                } else {
                    let pnl_money = codeset.unit_table as f64
                        * (trade.price.clone() - f.price.clone())
                        * f.amount.clone();
                    let pnl_ratio =
                        pnl_money / (f.price.clone() * f.amount.clone() * codeset.calc_coeff());
                    self.pair.push(QATradePair {
                        open_datetime: f.datetime.clone(),
                        close_datetime: trade.trade_date_time.clone(),
                        is_buy,
                        code: f.code.clone(),
                        amount: f.amount.clone(),
                        open_price: f.price.clone(),
                        close_price: trade.price.clone(),
                        open_trade_id: f.trade_id.clone(),
                        close_trade_id: trade.trade_id.clone(),
                        pnl_ratio,
                        pnl_money,
                    });
                    u.remove(0);
                }
            }
            _ => {}
        }
    }
    pub fn get_totalprofit(&mut self) -> f64 {
        let mut profit = 0.0;
        let _: Vec<_> = self
            .pair
            .iter_mut()
            .map(|a| profit += a.pnl_money)
            .collect();
        profit
    }
    ///
    /// 15%交易盈亏比：每天交易10次，，平均亏损，最大亏损
    /// 参考：交易实时盈亏比：引入行情，重点在评估每次操盘手平均冒多大风险，赚多大利润
    /// 15%胜率：多少次盈利，多少次亏损
    /// 40%绝对收益能力：通过操盘手收益（元）/日初总金额（万）。
    /// 30%资源周转能力：实际交易金额（元）/日初总金额（万）
    /// 手续费贡献：差额手续费（元）/日出总金额（万）
    pub fn get_maxprofit(&mut self) -> f64 {
        let mut profit: Vec<f64> = vec![];
        let _: Vec<_> = self
            .pair
            .iter_mut()
            .map(|a| profit.push(a.pnl_money))
            .collect();
        profit.iter().cloned().fold(0. / 0., f64::max)
    }
    pub fn get_averageprofit(&mut self) -> f64 {
        if self.pair.len() > 0 {
            self.get_totalprofit() / self.pair.len() as f64
        } else {
            0.0
        }
    }
    pub fn get_profitcount(&mut self) -> i32 {
        let mut count = 0;
        let _: Vec<_> = self
            .pair
            .iter_mut()
            .map(|a| {
                if a.pnl_money > 0.0 {
                    count += 1
                }
            })
            .collect();
        count
    }
    pub fn get_losscount(&mut self) -> i32 {
        let mut count = 0;
        let _: Vec<_> = self
            .pair
            .iter_mut()
            .map(|a| {
                if a.pnl_money < 0.0 {
                    count += 1
                }
            })
            .collect();
        count
    }
}

#[cfg(test)]
mod tests {
    use crate::qaaccount::QA_Account;

    use super::*;

    #[test]
    fn test_to_qifi() {
        let code = "rb2005";
        let mut p = QAPerformance::new();
        let mut acc = QA_Account::new("RustT01B2_RBL8", "test", "admin", 10000000.0, false, "real");
        acc.init_h(code);
        acc.sell_open(code, 10.0, "2020-01-20 09:30:22", 3500.0);
        acc.buy_open(code, 10.0, "2020-01-20 09:52:00", 3500.0);
        assert_eq!(acc.get_volume_short(code), 10.0);
        assert_eq!(acc.get_volume_long(code), 10.0);
        acc.buy_close(code, 10.0, "2020-01-20 10:22:00", 3600.0);
        acc.buy_open(code, 10.0, "2020-01-20 13:54:00", 3500.0);
        acc.buy_open(code, 10.0, "2020-01-20 13:55:00", 3510.0);

        acc.sell_close(code, 20.0, "2020-01-20 14:52:00", 3620.0);
        acc.buy_open(code, 20.0, "2020-01-21 13:54:00", 3500.0);
        acc.sell_close(code, 15.0, "2020-01-21 13:55:00", 3510.0);

        acc.sell_close(code, 5.0, "2020-01-21 14:52:00", 3420.0);
        println!("{:#?}", acc.dailytrades);
        for (_, i) in acc.dailytrades.iter_mut() {
            println!("{:#?}", i);
            p.insert_trade(i.to_owned());
        }
        println!("{:#?}", p.pair);
        println!("{}", p.get_totalprofit())
    }

    #[test]
    fn test_backtest() {
        let code = "rb2005";
        let mut p = QAPerformance::new();
        let mut acc = QA_Account::new(
            "RustT01B2_RBL8",
            "test",
            "admin",
            10000000.0,
            false,
            "backtest",
        );
        acc.init_h(code);
        acc.sell_open(code, 10.0, "2020-01-20 09:30:22", 3500.0);
        acc.buy_open(code, 10.0, "2020-01-20 09:52:00", 3500.0);
        assert_eq!(acc.get_volume_short(code), 10.0);
        assert_eq!(acc.get_volume_long(code), 10.0);
        acc.buy_close(code, 10.0, "2020-01-20 10:22:00", 3600.0);
        acc.buy_open(code, 10.0, "2020-01-20 13:54:00", 3500.0);
        acc.buy_open(code, 10.0, "2020-01-20 13:55:00", 3510.0);

        acc.sell_close(code, 20.0, "2020-01-20 14:52:00", 3620.0);
        acc.buy_open(code, 20.0, "2020-01-21 13:54:00", 3500.0);
        acc.sell_close(code, 15.0, "2020-01-21 13:55:00", 3510.0);

        acc.sell_close(code, 5.0, "2020-01-21 14:52:00", 3420.0);

        for i in acc.history.iter_mut() {
            p.insert_trade(i.to_qifitrade());
        }
        println!("{:#?}", p.pair);
        println!("{}", p.get_totalprofit())
    }

    #[test]
    fn test_pair() {
        let mut acc = QA_Account::new("test", "test", "admin", 1000000.0, false, "real");
        let code = "Z$002352";
        let mut p = QAPerformance::new();
        acc.sell_open(code, 1000.0, "2020-04-03 09:30:22", 46.33);
        acc.buy_open(code, 1000.0, "2020-04-03 09:52:00", 46.86);

        acc.buy_close(code, 1000.0, "2020-04-03 10:22:00", 47.34);
        acc.sell_close(code, 1000.0, "2020-04-03 10:22:00", 47.34);
        acc.buy_open(code, 1000.0, "2020-04-03 13:54:00", 47.1);
        acc.buy_open(code, 1000.0, "2020-04-03 13:55:00", 47.11);

        acc.sell_close(code, 2000.0, "2020-04-03 14:52:00", 47.17);

        // acc.buy_open(code, 2000.0, "2020-04-03 13:54:00", 47.1);
        // acc.sell_close(code, 1000.0, "2020-04-03 13:55:00", 47.11);
        //
        // acc.sell_close(code, 1000.0, "2020-04-03 14:52:00", 47.17);
        for (_, i) in acc.dailytrades.iter_mut() {
            println!("{:#?}", i);
            p.insert_trade(i.to_owned());
        }
        println!("{:#?}", p.pair);
        println!("{:#?}", p.get_maxprofit());
        println!("{:#?}", p.get_averageprofit());
    }
}
