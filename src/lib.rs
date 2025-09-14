pub mod ecs;
pub mod examples;
pub mod diffing;
pub mod core;
pub mod rendering;
pub mod input;
pub mod web_render_sample;

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