import { World, Transform, Sprite, input } from "longhorn";

export function onStart(world: World) {
  console.log("Hello from Longhorn v2!");

  // Create a simple entity
  world.spawn("Player")
    .with(Transform, { x: 640, y: 360 })
    .build();
}

export function onUpdate(world: World, dt: number) {
  const player = world.find("Player");
  if (!player) return;

  // Move player with touch
  if (input.justPressed()) {
    const pos = input.position();
    if (pos) {
      const transform = player.get(Transform);
      transform.x = pos.x;
      transform.y = pos.y;
    }
  }
}

export function onTouchStart(world: World, x: number, y: number) {
  console.log(`Touch at (${x}, ${y})`);
}
