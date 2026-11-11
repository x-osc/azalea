use azalea_client::WalkDirection;
use azalea_core::{direction::CardinalDirection, position::BlockPos};

use crate::pathfinder::{
    astar,
    costs::{CLIMB_DOWN_ONE_BLOCK_COST, CLIMB_UP_ONE_BLOCK_COST, WALK_ONE_BLOCK_COST},
    moves::{Edge, ExecuteCtx, IsReachedCtx, MoveData, MovesCtx, default_is_reached},
    player_pos_to_block_pos,
    positions::RelBlockPos,
    world::is_block_state_climbable,
};

pub fn climb_move(ctx: &mut MovesCtx, node: RelBlockPos) {
    if ctx.world.is_block_climbable(node) {
        climb_up_move(ctx, node);
        climb_down_move(ctx, node);
        climb_exit_move(ctx, node);
    }

    // disabled for now, untested
    // enter_ladder_move(ctx, pos);
    descend_into_ladder_move(ctx, node);
}

fn climb_up_move(ctx: &mut MovesCtx, pos: RelBlockPos) {
    let above = pos.up(1);
    if !ctx.world.is_block_climbable(above) {
        return;
    }
    // our feet will be in `above` so we need headroom there
    if !ctx.world.is_block_passable(above.up(1)) {
        return;
    }

    ctx.edges.push(Edge {
        movement: astar::Movement {
            target: above,
            data: MoveData {
                execute: &execute_climb_up_move,
                is_reached: &climb_is_reached,
            },
        },
        cost: CLIMB_UP_ONE_BLOCK_COST,
    })
}

fn execute_climb_up_move(mut ctx: ExecuteCtx) {
    let target = ctx.target;
    center_on_column(&mut ctx, target);
    ctx.jump();
}

fn climb_down_move(ctx: &mut MovesCtx, pos: RelBlockPos) {
    let below = pos.down(1);
    if !ctx.world.is_block_climbable(below) {
        return;
    }

    ctx.edges.push(Edge {
        movement: astar::Movement {
            target: below,
            data: MoveData {
                execute: &execute_climb_down_move,
                is_reached: &climb_is_reached,
            },
        },
        cost: CLIMB_DOWN_ONE_BLOCK_COST,
    })
}

fn execute_climb_down_move(mut ctx: ExecuteCtx) {
    let target = ctx.target;
    center_on_column(&mut ctx, target);
}

/// climb one block and then step off ladder onto a platform next to it
fn climb_exit_move(ctx: &mut MovesCtx, pos: RelBlockPos) {
    if !ctx.world.is_passable(pos.up(1)) {
        return;
    }

    for dir in CardinalDirection::iter() {
        let target = pos + RelBlockPos::new(dir.x(), 1, dir.z());
        if !ctx.world.is_standable(target) {
            continue;
        }

        ctx.edges.push(Edge {
            movement: astar::Movement {
                target,
                data: MoveData {
                    execute: &execute_climb_exit_move,
                    is_reached: &default_is_reached,
                },
            },
            cost: CLIMB_UP_ONE_BLOCK_COST + WALK_ONE_BLOCK_COST,
        });
    }
}

fn execute_climb_exit_move(mut ctx: ExecuteCtx) {
    let target = ctx.target;
    let position = ctx.position;
    let center = target.center();

    let over_target = player_pos_to_block_pos(position) == target;

    if over_target {
        let horizontal_distance = (center - position).horizontal_distance();
        if horizontal_distance > 0.25 {
            ctx.look_at(center);
            ctx.walk(WalkDirection::Forward);
        } else {
            ctx.walk(WalkDirection::None);
        }
    } else {
        ctx.look_at(center);
        ctx.walk(WalkDirection::Forward);

        // TODO: use on climbable from physics?
        let feet_block = ctx.get_block_state(BlockPos::from(position));
        if is_block_state_climbable(feet_block) {
            ctx.jump();
        }
    }
}

/// Walk sideways into a ladder at the same height.
///
/// if there's something to stand on under the ladder then `forward_move`
/// already generates this, so we skip these cases.
// TODO:
#[expect(dead_code)]
fn enter_ladder_move(ctx: &mut MovesCtx, pos: RelBlockPos) {
    for dir in CardinalDirection::iter() {
        let target = pos + RelBlockPos::new(dir.x(), 0, dir.z());

        if !ctx.world.is_block_climbable(target) {
            continue;
        }
        if ctx.world.is_block_standable(target.down(1)) {
            continue;
        }
        if !ctx.world.is_passable(target) {
            continue;
        }

        ctx.edges.push(Edge {
            movement: astar::Movement {
                target,
                data: MoveData {
                    execute: &execute_enter_ladder_move,
                    is_reached: &climb_is_reached,
                },
            },
            cost: WALK_ONE_BLOCK_COST,
        });
    }
}

fn descend_into_ladder_move(ctx: &mut MovesCtx, pos: RelBlockPos) {
    for dir in CardinalDirection::iter() {
        let offset = pos + RelBlockPos::new(dir.x(), 0, dir.z());
        let target = offset.down(1);

        if !ctx.world.is_block_climbable(target) {
            continue;
        }
        if !ctx.world.is_passable(offset) {
            continue;
        }

        ctx.edges.push(Edge {
            movement: astar::Movement {
                target,
                data: MoveData {
                    execute: &execute_enter_ladder_move,
                    is_reached: &climb_is_reached,
                },
            },
            cost: WALK_ONE_BLOCK_COST + CLIMB_DOWN_ONE_BLOCK_COST,
        })
    }
}

fn execute_enter_ladder_move(mut ctx: ExecuteCtx) {
    let target = ctx.target;
    ctx.look_at(target.center());
    ctx.walk(WalkDirection::Forward);
}

#[must_use]
pub fn climb_is_reached(
    IsReachedCtx {
        position, target, ..
    }: IsReachedCtx,
) -> bool {
    BlockPos::from(position) == target
}

fn center_on_column(ctx: &mut ExecuteCtx, column: BlockPos) {
    let center = column.center();
    let horizontal_distance = center.horizontal_distance_to(ctx.position);

    if horizontal_distance > 0.35 {
        ctx.look_at(center);
        ctx.walk(WalkDirection::Forward);
    } else {
        ctx.walk(WalkDirection::None)
    }
}
