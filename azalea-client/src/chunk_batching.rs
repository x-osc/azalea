//! Used for Minecraft's chunk batching introduced in 23w31a (1.20.2). It's used
//! for making the server spread out how often it sends us chunk packets
//! depending on our receiving speed.

use std::time::{Duration, Instant};

use azalea_protocol::packets::game::serverbound_chunk_batch_received_packet::ServerboundChunkBatchReceivedPacket;
use bevy_app::{App, Plugin, Update};
use bevy_ecs::prelude::*;

use crate::{
    interact::handle_block_interact_event,
    inventory::InventorySet,
    local_player::{handle_send_packet_event, SendPacketEvent},
    respawn::perform_respawn,
};

pub struct ChunkBatchingPlugin;
impl Plugin for ChunkBatchingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_chunk_batch_start_event,
                handle_chunk_batch_finished_event,
            )
                .chain()
                .before(handle_send_packet_event)
                .before(InventorySet)
                .before(handle_block_interact_event)
                .before(perform_respawn),
        )
        .add_event::<ChunkBatchStartEvent>()
        .add_event::<ChunkBatchFinishedEvent>();
    }
}

#[derive(Component, Clone, Debug)]
pub struct ChunkBatchInfo {
    pub start_time: Instant,
    pub aggregated_duration_per_chunk: Duration,
    pub old_samples_weight: u32,
}

impl ChunkBatchInfo {
    pub fn batch_finished(&mut self, batch_size: u32) {
        if batch_size == 0 {
            return;
        }
        let batch_duration = self.start_time.elapsed();
        let duration_per_chunk = batch_duration / batch_size;
        let clamped_duration = Duration::clamp(
            duration_per_chunk,
            self.aggregated_duration_per_chunk / 3,
            self.aggregated_duration_per_chunk * 3,
        );
        self.aggregated_duration_per_chunk =
            ((self.aggregated_duration_per_chunk * self.old_samples_weight) + clamped_duration)
                / (self.old_samples_weight + 1);
        self.old_samples_weight = u32::min(49, self.old_samples_weight + 1);
    }

    pub fn desired_chunks_per_tick(&self) -> f32 {
        (7000000. / self.aggregated_duration_per_chunk.as_nanos() as f64) as f32
    }
}

#[derive(Event)]
pub struct ChunkBatchStartEvent {
    pub entity: Entity,
}
#[derive(Event)]
pub struct ChunkBatchFinishedEvent {
    pub entity: Entity,
    pub batch_size: u32,
}

pub fn handle_chunk_batch_start_event(
    mut query: Query<&mut ChunkBatchInfo>,
    mut events: EventReader<ChunkBatchStartEvent>,
) {
    for event in events.read() {
        if let Ok(mut chunk_batch_info) = query.get_mut(event.entity) {
            chunk_batch_info.start_time = Instant::now();
        }
    }
}

pub fn handle_chunk_batch_finished_event(
    mut query: Query<&mut ChunkBatchInfo>,
    mut events: EventReader<ChunkBatchFinishedEvent>,
    mut send_packets: EventWriter<SendPacketEvent>,
) {
    for event in events.read() {
        if let Ok(mut chunk_batch_info) = query.get_mut(event.entity) {
            chunk_batch_info.batch_finished(event.batch_size);
            let desired_chunks_per_tick = chunk_batch_info.desired_chunks_per_tick();
            send_packets.send(SendPacketEvent {
                entity: event.entity,
                packet: ServerboundChunkBatchReceivedPacket {
                    desired_chunks_per_tick,
                }
                .get(),
            });
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChunkReceiveSpeedAccumulator {
    batch_sizes: Vec<u32>,
    /// as milliseconds
    batch_durations: Vec<u32>,
    index: usize,
    filled_size: usize,
}
impl ChunkReceiveSpeedAccumulator {
    pub fn new(capacity: usize) -> Self {
        Self {
            batch_sizes: vec![0; capacity],
            batch_durations: vec![0; capacity],
            index: 0,
            filled_size: 0,
        }
    }

    pub fn accumulate(&mut self, batch_size: u32, batch_duration: Duration) {
        self.batch_sizes[self.index] = batch_size;
        self.batch_durations[self.index] =
            f32::clamp(batch_duration.as_millis() as f32, 0., 15000.) as u32;
        self.index = (self.index + 1) % self.batch_sizes.len();
        if self.filled_size < self.batch_sizes.len() {
            self.filled_size += 1;
        }
    }

    pub fn get_millis_per_chunk(&self) -> f64 {
        let mut total_batch_size = 0;
        let mut total_batch_duration = 0;
        for i in 0..self.filled_size {
            total_batch_size += self.batch_sizes[i];
            total_batch_duration += self.batch_durations[i];
        }
        if total_batch_size == 0 {
            return 0.;
        }
        total_batch_duration as f64 / total_batch_size as f64
    }
}

impl Default for ChunkBatchInfo {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            aggregated_duration_per_chunk: Duration::from_millis(2),
            old_samples_weight: 1,
        }
    }
}
