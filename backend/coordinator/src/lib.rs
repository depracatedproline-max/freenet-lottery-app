// Coordinator library - modular subcomponents

pub mod task_manager;
pub mod worker_registry;
pub mod result_aggregator;
pub mod lottery_engine;
pub mod freenet_scaffold;

pub use task_manager::TaskManager;
pub use worker_registry::WorkerRegistry;
pub use result_aggregator::ResultAggregator;
pub use lottery_engine::LotteryEngine;
pub use freenet_scaffold::FreenetScaffoldState;
