ThisBuild / scalaVersion := "2.12.18"

val spinalVersion = "1.10.2"
libraryDependencies ++= Seq(
  "com.github.spinalhdl" %% "spinalhdl-core" % spinalVersion,
  "com.github.spinalhdl" %% "spinalhdl-lib" % spinalVersion
)
addCompilerPlugin(
  "com.github.spinalhdl" %% "spinalhdl-idsl-plugin" % spinalVersion
)
Compile / run / fork := true
Compile / run / javaOptions ++= Seq("-Xms512m", "-Xmx2g")
