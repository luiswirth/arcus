
// Hardware task, bound to a hardware interrupt
#[task(
      binds = TIMER_IRQ_0,
      priority = 1,
      shared = [timer, alarm, led],
      local = [tog: bool = true],
  )]
fn timer_irq(mut c: timer_irq::Context) {
}
