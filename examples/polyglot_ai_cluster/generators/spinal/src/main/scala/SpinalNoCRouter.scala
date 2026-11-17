import spinal.core._

class SpinalNoCRouter extends Component {
  val io = new Bundle {
    val route = in UInt (2 bits)
    val flit = in Bits (8 bits)
    val west = out Bits (8 bits)
    val east = out Bits (8 bits)
    val north = out Bits (8 bits)
    val south = out Bits (8 bits)
  }

  io.west := 0
  io.east := 0
  io.north := 0
  io.south := 0

  switch(io.route) {
    is(U(0, 2 bits)) { io.west := io.flit }
    is(U(1, 2 bits)) { io.east := io.flit }
    is(U(2, 2 bits)) { io.north := io.flit }
    is(U(3, 2 bits)) { io.south := io.flit }
  }
}

object GenerateSpinalNoCRouter extends App {
  SpinalConfig(targetDirectory = "../../rtl/generated").generateVerilog(new SpinalNoCRouter)
}
