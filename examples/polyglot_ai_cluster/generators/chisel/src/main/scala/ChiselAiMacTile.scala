import chisel3._
import circt.stage.ChiselStage

class ChiselAiMacTile extends Module {
  val io = IO(new Bundle {
    val a = Input(UInt(8.W))
    val b = Input(UInt(8.W))
    val acc = Input(UInt(16.W))
    val y = Output(UInt(16.W))
  })

  io.y := io.a * io.b + io.acc
}

object GenerateChiselAiMacTile extends App {
  val output = "../../rtl/generated"
  ChiselStage.emitSystemVerilog(
    new ChiselAiMacTile,
    Array("--target-dir", output)
  )
}
