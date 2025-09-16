#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::ospi::{AddressSize, Ospi, OspiError, OspiWidth, TransferConfig};
use embassy_stm32::peripherals;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Testing OSPI error handling");

    let peri = embassy_stm32::init(global_config());

    let qspi_config = embassy_stm32::ospi::Config {
        device_size: embassy_stm32::ospi::MemorySize::_1KiB,
        ..Default::default()
    };

    let mut driver: Ospi<'_, peripherals::OCTOSPI1, embassy_stm32::mode::Async> = Ospi::new_quadspi(
        peri.OCTOSPI1,
        // Clock
        peri.PB10,
        peri.PE12,
        peri.PB0,
        peri.PE14,
        peri.PE15,
        // CS pin aka "negative slave select"
        peri.PA2,
        peri.GPDMA1_CH0,
        qspi_config,
    );

    info!("QSPI driver setup");

    // ####################
    // Valid transfers - using address and data length that fit into device_size.
    // ####################

    let transfer_config = TransferConfig {
        iwidth: OspiWidth::SING,
        instruction: Some(0xAA),
        isize: AddressSize::_8Bit,

        adwidth: OspiWidth::SING,
        address: Some(0x0),
        adsize: AddressSize::_32bit,

        ..Default::default()
    };

    let data = [0u8; 16];

    // TODO(goodhoko): test the valid reads using some real SPI device providing the data.

    crate::assert!(matches!(
        driver.blocking_command(&transfer_config),
        Result::Ok(())
    ));
    // crate::assert!(matches!(
    //     driver.blocking_read(&mut data, transfer_config),
    //     Result::Ok(())
    // ));
    crate::assert!(matches!(
        driver.blocking_write(&data, transfer_config),
        Result::Ok(())
    ));

    // crate::assert!(matches!(
    //     driver.blocking_read_dma(&mut data, transfer_config),
    //     Result::Ok(())
    // ));
    crate::assert!(matches!(
        driver.blocking_write_dma(&data, transfer_config),
        Result::Ok(())
    ));

    // crate::assert!(matches!(
    //     driver.read(&mut data, transfer_config).await,
    //     Result::Ok(())
    // ));
    crate::assert!(matches!(
        driver.write(&data, transfer_config).await,
        Result::Ok(())
    ));

    // ####################
    // Address out of range
    // ####################

    let transfer_config = TransferConfig {
        iwidth: OspiWidth::SING,
        instruction: Some(0xAA),
        isize: AddressSize::_8Bit,

        adwidth: OspiWidth::SING,
        // ---------💥👇
        address: Some(0xffffff),
        adsize: AddressSize::_32bit,

        ..Default::default()
    };

    let mut data = [0u8; 16];

    crate::assert!(matches!(
        driver.blocking_command(&transfer_config),
        Result::Err(OspiError::InvalidCommand)
    ));
    crate::assert!(matches!(
        driver.blocking_read(&mut data, transfer_config),
        Result::Err(OspiError::InvalidCommand)
    ));
    crate::assert!(matches!(
        driver.blocking_write(&data, transfer_config),
        Result::Err(OspiError::InvalidCommand)
    ));

    crate::assert!(matches!(
        driver.blocking_read_dma(&mut data, transfer_config),
        Result::Err(OspiError::InvalidCommand)
    ));
    crate::assert!(matches!(
        driver.blocking_write_dma(&data, transfer_config),
        Result::Err(OspiError::InvalidCommand)
    ));

    crate::assert!(matches!(
        driver.read(&mut data, transfer_config).await,
        Result::Err(OspiError::InvalidCommand)
    ));
    crate::assert!(matches!(
        driver.write(&data, transfer_config).await,
        Result::Err(OspiError::InvalidCommand)
    ));

    // ####################
    // Data length out of range
    // ####################

    let transfer_config = TransferConfig {
        iwidth: OspiWidth::SING,
        instruction: Some(0xAA),
        isize: AddressSize::_8Bit,

        adwidth: OspiWidth::SING,
        address: Some(0x0),
        adsize: AddressSize::_32bit,

        ..Default::default()
    };

    //--------------------------💥👇
    let mut data = [0u8; 2048];

    crate::assert!(matches!(
        driver.blocking_read(&mut data, transfer_config),
        Result::Err(OspiError::InvalidCommand)
    ));
    crate::assert!(matches!(
        driver.blocking_write(&data, transfer_config),
        Result::Err(OspiError::InvalidCommand)
    ));

    crate::assert!(matches!(
        driver.blocking_read_dma(&mut data, transfer_config),
        Result::Err(OspiError::InvalidCommand)
    ));
    crate::assert!(matches!(
        driver.blocking_write_dma(&data, transfer_config),
        Result::Err(OspiError::InvalidCommand)
    ));

    crate::assert!(matches!(
        driver.read(&mut data, transfer_config).await,
        Result::Err(OspiError::InvalidCommand)
    ));
    crate::assert!(matches!(
        driver.write(&data, transfer_config).await,
        Result::Err(OspiError::InvalidCommand)
    ));

    info!("ALL TESTS PASS");

    loop {}
}

fn global_config() -> embassy_stm32::Config {
    use embassy_stm32::Config;
    use embassy_stm32::rcc::*;
    use embassy_stm32::time::Hertz;

    let mut config = Config::default();

    config.rcc.hse = Some(Hse {
        freq: Hertz(16_000_000),
        mode: HseMode::Oscillator,
    });
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSE,
        // HSE / 2 = 8MHz
        prediv: PllPreDiv::DIV2,
        // 8MHz * 60 = 480MHz
        mul: PllMul::MUL60,
        // 480MHz / 3 = 160MHz (sys_ck)
        divr: Some(PllDiv::DIV3),
        // 480MHz / 4 = 120MHz (OctoSPI)
        divq: Some(PllDiv::DIV10), // DIV24 is the max I could do
        // 480MHz / 15 = 32MHz (USBOTG)
        divp: Some(PllDiv::DIV15),
    });

    config.rcc.sys = Sysclk::PLL1_R;
    config.rcc.mux.octospisel = embassy_stm32::rcc::mux::Octospisel::PLL1_Q;

    config.rcc.voltage_range = VoltageScale::RANGE1;

    config
}
