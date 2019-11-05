import scala.io.Source
import java.io.File
import java.io.PrintWriter
import scala.math._
import scala.collection.mutable.ArrayBuffer

object Main {
  def main(args: Array[String]): Unit = {
    if (args.length < 3) {
      println("Usage: scala SRC sbox linear_tab differential_tab")
      return
    }

    val sboxFile = args(0)
    val linearTabOut = args(1)
    val differentialTabOut = args(2)

    val sbox = readSboxTable(sboxFile)
    val linearTab = genLinearAnalysisTable(sbox)
    val differentialTab = genDifferentialAnalysisTable(sbox)

    writeTable(linearTab, linearTabOut)
    writeTable(differentialTab, differentialTabOut)
  }

  private def readSboxTable(path: String): Array[Array[Int]] = {
    Source
      .fromFile(path)
      .mkString
      .trim
      .split("\n")
      .map(line => line.trim.split(" ").map(x => Integer.parseInt(x.trim, 16)))
  }

  private def serializeTable(tab: Array[Array[Int]]): String = {
    val lineStrs = for {
      line <- tab
      lineStr = line.map(_.toString).mkString(" ")
    } yield lineStr
    lineStrs.mkString("\n")
  }

  private def writeTable(tab: Array[Array[Int]], path: String): Unit = {
    val writer = new PrintWriter(new File(path))
    writer.write(serializeTable(tab))
    writer.close
  }

  private def sboxSubstitute(in: Int)(implicit sbox: Array[Array[Int]]): Int = {
    if (sbox.length > 16) {
      throw new IllegalArgumentException("sbox cannot have more than 32 bits")
    }
    val maskBits = (log(sbox.length) / log(2)).toInt
    val low = in & ((1 << maskBits) - 1)
    val high = in >> maskBits
    sbox(low)(high)
  }

  private def defaultTable(size: Int): Array[Array[Int]] = {
    val buf = ArrayBuffer.empty[Array[Int]]

    for (i <- 0 to (size * size - 1)) {
      val tmp = ArrayBuffer.empty[Int]
      for (_ <- 0 to (size * size - 1)) {
        tmp += 0
      }
      buf += tmp.toArray
    }

    buf.toArray
  }

  private def bitPilingXor(num: Int): Int = {
    var x = 0
    var numVar = num
    while (numVar > 0) {
      x ^= numVar & 1
      numVar >>= 1
    }
    x
  }

  private def linearApprox(in: Int, out: Int)(implicit sbox: Array[Array[Int]]): Int = {
    var count = 0
    for (i <- 0 to (sbox.length * sbox.length - 1)) {
      val inMasked = in & i
      val outMasked = out & sboxSubstitute(i)
      if (bitPilingXor(inMasked) == bitPilingXor(outMasked)) {
        count += 1
      }
    }
    count
  }

  private def genLinearAnalysisTable(sbox: Array[Array[Int]]): Array[Array[Int]] = {
    implicit val impSbox = sbox
    val tab = defaultTable(sbox.length)
    for {
      in <- 0 to (sbox.length * sbox.length - 1)
      out <- 0 to (sbox.length * sbox.length - 1)
    } {
      tab(in)(out) = linearApprox(in, out)
    }
    return tab
  }

  private def genDifferentialAnalysisTable(sbox: Array[Array[Int]]): Array[Array[Int]] = {
    implicit val impSbox = sbox
    val tab = defaultTable(sbox.length)
    for {
      first <- 0 to (sbox.length * sbox.length - 1)
      second <- 0 to (sbox.length * sbox.length - 1)
    } {
      val diff = first ^ second
      val subFirst = sboxSubstitute(first)
      val subSecond = sboxSubstitute(second)
      val outDiff = subFirst ^ subSecond
      tab(diff)(outDiff) += 1
    }

    tab
  }
}
