#![no_main]
use libfuzzer_sys::fuzz_target;

struct StaticSymbolSupplier {
    file: Vec<u8>,
}

#[async_trait::async_trait]
impl breakpad_symbols::SymbolSupplier for StaticSymbolSupplier {
    async fn locate_symbols(
        &self,
        _module: &(dyn minidump_common::traits::Module + Sync),
    ) -> Result<breakpad_symbols::SymbolFile, breakpad_symbols::SymbolError> {
        breakpad_symbols::SymbolFile::from_bytes(&self.file)
    }
    async fn locate_file(
        &self,
        _module: &(dyn minidump_common::traits::Module + Sync),
        _file_kind: breakpad_symbols::FileKind,
    ) -> Result<std::path::PathBuf, breakpad_symbols::FileError> {
        Err(breakpad_symbols::FileError::NotFound)
    }
}

fuzz_target!(|data: (&[u8], &[u8])| {
    if let Ok(dump) = minidump::Minidump::read(data.0) {
        let supplier = StaticSymbolSupplier {
            file: data.1.to_vec(),
        };

        let provider = breakpad_symbols::Symbolizer::new(supplier);
        // Fuzz every possible feature
        let options = minidump_processor::ProcessorOptions::unstable_all();

        let val: Result<_, _> = minidump_processor_fuzz::fuzzing_block_on(
            minidump_processor::process_minidump_with_options(&dump, &provider, options),
        );

        if let Ok(v) = val {
            let _: Result<(), _> = v.print_json(&mut std::io::sink(), true);
        }
    }
});
