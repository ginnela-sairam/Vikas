ThisBuild / scalaVersion := "2.13.12"

libraryDependencies += "org.chipsalliance" %% "chisel" % "6.5.0"
addCompilerPlugin(
  "org.chipsalliance" % "chisel-plugin" % "6.5.0" cross CrossVersion.full
)

Compile / run / fork := true
