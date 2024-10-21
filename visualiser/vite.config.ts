import { defineConfig } from "vite"
import fs from "fs/promises"
import path from "path"
import type { Plugin } from "vite"

interface TreeFile {
  id: string
  name: string
}

interface TreeNode {
  id: string
  body: string
  visits: number
  avg_score: number
  ucb: number
  isMostVisited: boolean
  children?: TreeNode[]
  board: Board
}
interface Board {
  height: number
  width: number
  food: Point[]
  hazards: Point[]
  snakes: Snake[]
}

interface Point {
  x: number
  y: number
}

interface Snake {
  id: string
  name: string
  health: number
  body: Point[]
  latency: string
  head: Point
  shout: string
}

const treeDataPlugin: Plugin = {
  name: "tree-data-plugin",
  configureServer(server) {
    server.middlewares.use(async (req, res, next) => {
      if (req.url?.startsWith("/api/trees")) {
        const parts = req.url.split("/")
        const fileName = parts[parts.length - 1]

        try {
          if (fileName === "trees") {
            // Serve directory list
            const treeDataPath = "./tree-data"
            const files = await fs.readdir(treeDataPath)
            const treeFiles: TreeFile[] = files.map((file) => ({
              id: path.parse(file).name, // Use filename without extension as id
              name: file,
            }))
            res.setHeader("Content-Type", "application/json")
            return res.end(JSON.stringify(treeFiles))
          } else {
            // Serve JSON file contents
            const filePath = path.join("./tree-data", fileName)
            const data = await fs.readFile(filePath, "utf8")
            const treeNode: TreeNode = JSON.parse(data)

            // Validate that the file content matches the TreeNode interface
            if (!isValidTreeNode(treeNode)) {
              res.statusCode = 400
              return res.end(JSON.stringify({ error: "Invalid TreeNode data" }))
            }

            res.setHeader("Content-Type", "application/json")
            return res.end(JSON.stringify(treeNode))
          }
        } catch (error) {
          console.error("Error:", error)
          res.statusCode = 500
          return res.end(JSON.stringify({ error: "Internal Server Error" }))
        }
      }
      next()
    })
  },
}

// Helper function to validate TreeNode structure
function isValidTreeNode(node: any): node is TreeNode {
  return (
    typeof node.id === "string" &&
    typeof node.body === "string" &&
    typeof node.visits === "number" &&
    typeof node.avg_score === "number" &&
    typeof node.ucb === "number" &&
    typeof node.isMostVisited === "boolean" &&
    (node.children === undefined || Array.isArray(node.children))
  )
}

export default defineConfig({
  plugins: [treeDataPlugin],
})
