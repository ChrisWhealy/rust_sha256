use super::*;

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_choose_correctly() -> Result<(), String> {
    let selector: u32 = 0xFF_00_FF_00;
    let when_hi: u32 = 0xFF_00_FF_00;
    let when_lo: u32 = 0x00_FF_00_FF;
    let ans: u32 = 0xFF_FF_FF_FF;

    if choose(selector, when_hi, when_lo) != ans {
        return Err(format!(
            "choose() returned 0x{:08X}, expected 0x{:08X}",
            choose(selector, when_hi, when_lo), ans
        ));
    }

    let selector: u32 = 0xFF_00_FF_00;
    let when_hi: u32 = 0x00_FF_00_FF;
    let when_lo: u32 = 0xFF_00_FF_00;
    let ans: u32 = 0x00_00_00_00;

    if choose(selector, when_hi, when_lo) != ans {
        return Err(format!(
            "choose() returned 0x{:08X}, expected 0x{:08X}",
            choose(selector, when_hi, when_lo), ans
        ));
    }

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -
#[test]
fn should_calculate_majority() -> Result<(), String> {
    let a: u32 = 0xFF_00_00_00;
    let b: u32 = 0x00_FF_00_00;
    let c: u32 = 0x00_00_FF_00;
    let ans: u32 = 0x00_00_00_00;

    if majority(a, b, c) != ans {
        return Err(format!(
            "majority() returned 0x{:08X}, expected {:08X}",
            majority(a, b, c), ans
        ));
    }

    let a: u32 = 0xFF_FF_00_00;
    let b: u32 = 0xFF_00_FF_00;
    let c: u32 = 0x00_FF_FF_00;
    let ans: u32 = 0xFF_FF_FF_00;

    if majority(a, b, c) != ans {
        return Err(format!(
            "majority() returned 0x{:08X}, expected {:08X}",
            majority(a, b, c), ans
        ));
    }

    let a: u32 = 0xFF_FF_FF_00;
    let b: u32 = 0xFF_FF_FF_00;
    let c: u32 = 0xFF_FF_FF_00;
    let ans: u32 = 0xFF_FF_FF_00;

    if majority(a, b, c) != ans {
        return Err(format!(
            "majority() returned 0x{:08X}, expected {ans}",
            majority(a, b, c)
        ));
    }

    Ok(())
}
