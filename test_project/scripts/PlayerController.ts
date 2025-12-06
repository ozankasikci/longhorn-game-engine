// PlayerController.ts - Test script for Longhorn engine
export default class PlayerController {
  speed = 100;

  onStart(self: any) {
    console.log("Starting at position:", self.transform.position.x, self.transform.position.y);
  }

  onUpdate(self: any, dt: number) {
    // Move right
    self.transform.position.x += this.speed * dt;
  }
}
