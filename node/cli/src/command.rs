use crate::cli::{Cli, Subcommand};
use frame_benchmarking_cli::BenchmarkCmd;
use node_inspect::cli::InspectCmd;
use sc_cli::{
	ChainSpec, CheckBlockCmd, ExportBlocksCmd, ExportStateCmd, ImportBlocksCmd, Result, RevertCmd,
	RuntimeVersion, SubstrateCli,
};
use sc_service::PartialComponents;
use ternoa_service::{chain_spec, new_full, new_partial, IdentifyVariant};

#[cfg(feature = "chaosnet-native")]
use ternoa_service::chaosnet_runtime;
#[cfg(feature = "chaosnet-native")]
use ternoa_service::ChaosnetExecutorDispatch;

#[cfg(feature = "alphanet-native")]
use ternoa_service::alphanet_runtime;
#[cfg(feature = "alphanet-native")]
use ternoa_service::AlphanetExecutorDispatch;

#[cfg(feature = "mainnet-native")]
use ternoa_service::mainnet_runtime;
#[cfg(feature = "mainnet-native")]
use ternoa_service::MainnetExecutorDispatch;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Ternoa Node".into()
	}

	fn impl_version() -> String {
		env!("CARGO_PKG_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/capsule-corp-ternoa/chain/issues".into()
	}

	fn copyright_start_year() -> i32 {
		2020
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"chaosnet" => Box::new(chain_spec::chaosnet_config()?),
			#[cfg(feature = "chaosnet-native")]
			"chaosnet-dev" | "c-dev" => Box::new(chain_spec::chaosnet::development_config()),

			"alphanet" => Box::new(chain_spec::alphanet_config()?),
			#[cfg(feature = "alphanet-native")]
			"alphanet-dev" | "a-dev" => Box::new(chain_spec::alphanet::development_config()),
			#[cfg(feature = "alphanet-native")]
			"alphanet-sta" | "a-sta" => Box::new(chain_spec::alphanet::staging_config()),

			"mainnet" => Box::new(chain_spec::mainnet_config()?),
			#[cfg(feature = "mainnet-native")]
			"mainnet-dev" | "dev" => Box::new(chain_spec::mainnet::development_config()),

			"" => return Err("Please specify which chain you want to run!".into()),
			path => {
				let path = std::path::PathBuf::from(path);

				let chain_spec =
					Box::new(chain_spec::MainnetChainSpec::from_json_file(path.clone())?)
						as Box<dyn sc_service::ChainSpec>;

				if chain_spec.is_chaosnet() {
					Box::new(chain_spec::ChaosnetChainSpec::from_json_file(path)?)
				} else if chain_spec.is_alphanet() {
					Box::new(chain_spec::AlphanetChainSpec::from_json_file(path)?)
				} else {
					chain_spec
				}
			},
		})
	}

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		#[cfg(feature = "chaosnet-native")]
		if spec.is_chaosnet() {
			return &chaosnet_runtime::VERSION
		}

		#[cfg(feature = "alphanet-native")]
		if spec.is_alphanet() {
			return &alphanet_runtime::VERSION
		}

		#[cfg(feature = "mainnet-native")]
		{
			return &mainnet_runtime::VERSION
		}

		#[cfg(not(feature = "mainnet-native"))]
		panic!("No runtime feature (chaosnet, alphanet, mainnet) is enabled");
	}
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	// When we call cli.create_runner() it automatically calls the cli.load_spec() function. The
	// loaded spec is stored inside runner.config().chain_spec.

	match &cli.subcommand {
		None => run_wo_args(&cli),
		Some(Subcommand::Inspect(cmd)) => inspect(&cli, cmd),
		Some(Subcommand::Benchmark(cmd)) => benchmark(&cli, cmd),
		Some(Subcommand::Key(cmd)) => Ok(cmd.run(&cli)?),
		Some(Subcommand::Sign(cmd)) => Ok(cmd.run()?),
		Some(Subcommand::Verify(cmd)) => Ok(cmd.run()?),
		Some(Subcommand::Vanity(cmd)) => Ok(cmd.run()?),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			Ok(runner.sync_run(|config| cmd.run(config.chain_spec, config.network))?)
		},
		Some(Subcommand::CheckBlock(cmd)) => check_block(&cli, cmd),
		Some(Subcommand::ExportBlocks(cmd)) => export_blocks(&cli, cmd),
		Some(Subcommand::ExportState(cmd)) => export_state(&cli, cmd),
		Some(Subcommand::ImportBlocks(cmd)) => import_blocks(&cli, cmd),
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			Ok(runner.sync_run(|config| cmd.run(config.database))?)
		},
		Some(Subcommand::Revert(cmd)) => revert(&cli, cmd),
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				// we don't need any of the components of new_partial, just a runtime, or a task
				// manager to do `async_run`.
				let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager =
					sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
						.map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;

				Ok((cmd.run::<Block, ExecutorDispatch>(config), task_manager))
			})
		},
		#[cfg(not(feature = "try-runtime"))]
		Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
							 You can enable it with `--features try-runtime`."
			.into()),
	}?;

	Ok(())
}

fn run_wo_args(cli: &Cli) -> std::result::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(&cli.run)?;
	let chain_spec = &runner.config().chain_spec;

	#[cfg(feature = "chaosnet-native")]
	if chain_spec.is_chaosnet() {
		return Ok(runner.run_node_until_exit(|config| async move {
			new_full::<chaosnet_runtime::RuntimeApi, ChaosnetExecutorDispatch>(config)
				.map_err(sc_cli::Error::Service)
		})?)
	}

	#[cfg(feature = "alphanet-native")]
	if chain_spec.is_alphanet() {
		return Ok(runner.run_node_until_exit(|config| async move {
			new_full::<alphanet_runtime::RuntimeApi, AlphanetExecutorDispatch>(config)
				.map_err(sc_cli::Error::Service)
		})?)
	}

	#[cfg(feature = "mainnet-native")]
	{
		return Ok(runner.run_node_until_exit(|config| async move {
			new_full::<mainnet_runtime::RuntimeApi, MainnetExecutorDispatch>(config)
				.map_err(sc_cli::Error::Service)
		})?)
	}

	#[cfg(not(feature = "mainnet-native"))]
	panic!("No runtime feature (chaosnet, alphanet, mainnet) is enabled");
}

fn inspect(cli: &Cli, cmd: &InspectCmd) -> std::result::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(cmd)?;
	let chain_spec = &runner.config().chain_spec;

	#[cfg(feature = "chaosnet-native")]
	if chain_spec.is_chaosnet() {
		return Ok(runner.sync_run(|config| {
			cmd.run::<chaosnet_runtime::Block, chaosnet_runtime::RuntimeApi, ChaosnetExecutorDispatch>(config)
		})?)
	}

	#[cfg(feature = "alphanet-native")]
	if chain_spec.is_alphanet() {
		return Ok(runner.sync_run(|config| {
			cmd.run::<alphanet_runtime::Block, alphanet_runtime::RuntimeApi, AlphanetExecutorDispatch>(config)
		})?)
	}

	#[cfg(feature = "mainnet-native")]
	{
		return Ok(runner.sync_run(|config| {
			cmd.run::<mainnet_runtime::Block, mainnet_runtime::RuntimeApi, MainnetExecutorDispatch>(
				config,
			)
		})?)
	}

	#[cfg(not(feature = "mainnet-native"))]
	panic!("No runtime feature (chaosnet, alphanet, mainnet) is enabled")
}

fn benchmark(cli: &Cli, cmd: &BenchmarkCmd) -> std::result::Result<(), sc_cli::Error> {
	if !cfg!(feature = "runtime-benchmarks") {
		return Err("Benchmarking wasn't enabled when building the node. \
					 You can enable it with `--features runtime-benchmarks`."
			.into())
	}

	let runner = cli.create_runner(cmd)?;
	let chain_spec = &runner.config().chain_spec;

	#[cfg(feature = "chaosnet-native")]
	if chain_spec.is_chaosnet() {
		return Ok(runner.sync_run(|config| {
			cmd.run::<chaosnet_runtime::Block, ChaosnetExecutorDispatch>(config)
		})?)
	}

	#[cfg(feature = "alphanet-native")]
	if chain_spec.is_alphanet() {
		return Ok(runner.sync_run(|config| {
			cmd.run::<alphanet_runtime::Block, AlphanetExecutorDispatch>(config)
		})?)
	}

	#[cfg(feature = "mainnet-native")]
	{
		return Ok(runner.sync_run(|config| {
			cmd.run::<mainnet_runtime::Block, MainnetExecutorDispatch>(config)
		})?)
	}

	#[cfg(not(feature = "mainnet-native"))]
	panic!("No runtime feature (chaosnet, alphanet, mainnet) is enabled")
}

fn check_block(cli: &Cli, cmd: &CheckBlockCmd) -> std::result::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(cmd)?;
	let chain_spec = &runner.config().chain_spec;

	#[cfg(feature = "chaosnet-native")]
	if chain_spec.is_chaosnet() {
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, import_queue, .. } =
				new_partial::<chaosnet_runtime::RuntimeApi, ChaosnetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, import_queue), task_manager))
		})?)
	}

	#[cfg(feature = "alphanet-native")]
	if chain_spec.is_alphanet() {
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, import_queue, .. } =
				new_partial::<alphanet_runtime::RuntimeApi, AlphanetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, import_queue), task_manager))
		})?)
	}

	#[cfg(feature = "mainnet-native")]
	{
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, import_queue, .. } =
				new_partial::<mainnet_runtime::RuntimeApi, MainnetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, import_queue), task_manager))
		})?)
	}

	#[cfg(not(feature = "mainnet-native"))]
	panic!("No runtime feature (chaosnet, alphanet, mainnet) is enabled");
}

fn export_blocks(cli: &Cli, cmd: &ExportBlocksCmd) -> std::result::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(cmd)?;
	let chain_spec = &runner.config().chain_spec;

	#[cfg(feature = "chaosnet-native")]
	if chain_spec.is_chaosnet() {
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, .. } =
				new_partial::<chaosnet_runtime::RuntimeApi, ChaosnetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, config.database), task_manager))
		})?)
	}

	#[cfg(feature = "alphanet-native")]
	if chain_spec.is_alphanet() {
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, .. } =
				new_partial::<alphanet_runtime::RuntimeApi, AlphanetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, config.database), task_manager))
		})?)
	}

	#[cfg(feature = "mainnet-native")]
	{
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, .. } =
				new_partial::<mainnet_runtime::RuntimeApi, MainnetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, config.database), task_manager))
		})?)
	}

	#[cfg(not(feature = "mainnet-native"))]
	panic!("No runtime feature (chaosnet, alphanet, mainnet) is enabled");
}

fn import_blocks(cli: &Cli, cmd: &ImportBlocksCmd) -> std::result::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(cmd)?;
	let chain_spec = &runner.config().chain_spec;

	#[cfg(feature = "chaosnet-native")]
	if chain_spec.is_chaosnet() {
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, import_queue, .. } =
				new_partial::<chaosnet_runtime::RuntimeApi, ChaosnetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, import_queue), task_manager))
		})?)
	}

	#[cfg(feature = "alphanet-native")]
	if chain_spec.is_alphanet() {
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, import_queue, .. } =
				new_partial::<alphanet_runtime::RuntimeApi, AlphanetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, import_queue), task_manager))
		})?)
	}

	#[cfg(feature = "mainnet-native")]
	{
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, import_queue, .. } =
				new_partial::<mainnet_runtime::RuntimeApi, MainnetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, import_queue), task_manager))
		})?)
	}

	#[cfg(not(feature = "mainnet-native"))]
	panic!("No runtime feature (chaosnet, alphanet, mainnet) is enabled");
}

fn revert(cli: &Cli, cmd: &RevertCmd) -> std::result::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(cmd)?;
	let chain_spec = &runner.config().chain_spec;

	#[cfg(feature = "chaosnet-native")]
	if chain_spec.is_chaosnet() {
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, backend, .. } =
				new_partial::<chaosnet_runtime::RuntimeApi, ChaosnetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, backend, None), task_manager))
		})?)
	}

	#[cfg(feature = "alphanet-native")]
	if chain_spec.is_alphanet() {
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, backend, .. } =
				new_partial::<alphanet_runtime::RuntimeApi, AlphanetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, backend, None), task_manager))
		})?)
	}

	#[cfg(feature = "mainnet-native")]
	{
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, backend, .. } =
				new_partial::<mainnet_runtime::RuntimeApi, MainnetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, backend, None), task_manager))
		})?)
	}

	#[cfg(not(feature = "mainnet-native"))]
	panic!("No runtime feature (chaosnet, alphanet, mainnet) is enabled");
}

fn export_state(cli: &Cli, cmd: &ExportStateCmd) -> std::result::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(cmd)?;
	let chain_spec = &runner.config().chain_spec;

	#[cfg(feature = "chaosnet-native")]
	if chain_spec.is_chaosnet() {
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, .. } =
				new_partial::<chaosnet_runtime::RuntimeApi, ChaosnetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, config.chain_spec), task_manager))
		})?)
	}

	#[cfg(feature = "alphanet-native")]
	if chain_spec.is_alphanet() {
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, .. } =
				new_partial::<alphanet_runtime::RuntimeApi, AlphanetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, config.chain_spec), task_manager))
		})?)
	}

	#[cfg(feature = "mainnet-native")]
	{
		return Ok(runner.async_run(|config| {
			let PartialComponents { client, task_manager, .. } =
				new_partial::<mainnet_runtime::RuntimeApi, MainnetExecutorDispatch>(&config)?;
			return Ok((cmd.run(client, config.chain_spec), task_manager))
		})?)
	}

	#[cfg(not(feature = "mainnet-native"))]
	panic!("No runtime feature (chaosnet, alphanet, mainnet) is enabled");
}
