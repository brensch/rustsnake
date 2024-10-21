import chokidar from "chokidar"
import fs from "fs"
import path from "path"

const dataFolder = "./tree-data" // Folder to monitor for tree files

// Initialize watcher to monitor folder
const watcher = chokidar.watch(dataFolder, {
  persistent: true,
  ignoreInitial: false,
})

// Event for when a new tree is added
watcher.on("add", (filePath) => {
  console.log(`New tree added: ${filePath}`)
})

// Event for when a tree is changed
watcher.on("change", (filePath) => {
  console.log(`Tree updated: ${filePath}`)
})

// Event for when a tree is deleted
watcher.on("unlink", (filePath) => {
  console.log(`Tree deleted: ${filePath}`)
})

// Utility function to load tree data
function loadTree(filePath: string) {
  return JSON.parse(fs.readFileSync(filePath, "utf-8"))
}
