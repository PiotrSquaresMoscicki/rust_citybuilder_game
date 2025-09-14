pub mod ecs;
pub mod examples;
pub mod diffing;
pub mod core;
pub mod rendering;
pub mod input;
pub mod web_render_sample;
pub mod time_demo;
pub mod dependency_example;

#[cfg(test)]
mod main_test;

#[cfg(test)]
mod diffing_test;

#[cfg(test)]
mod multiple_iterators_test;

#[cfg(test)]
mod multiple_iterator_systems_test;

#[cfg(test)]
mod rendering_test;

#[cfg(test)]
mod input_test;

#[cfg(test)]
mod input_demo;

#[cfg(test)]
mod system_dependency_test;

#[cfg(test)]
mod hierarchy_demo;