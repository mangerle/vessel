import { z } from 'zod'

/**
 * settings.json 物理文件的 zod schema
 *
 * 仅用于 loadSettings 时做一道防火墙：当用户手动编辑了配置文件、或老版本字段格式
 * 与新版不兼容时，不让坏数据直接污染 store，转为返回安全默认值并 console.error。
 *
 * 设计取舍：
 * - 字段全部 optional，因为 settings.json 是渐进式增长的，缺字段是常态；
 * - 老版本 legacy 字段（connectionMode/wslDistro/sshHost/...）也允许存在，
 *   loadSettings 中按需读取即可；
 * - connections / registries 内层用 strict object，避免被 user 不小心写入垃圾键。
 */

const ThemeSchema = z.enum(['deep-black', 'zed-gray', 'light-apple'])

const ConnectionTypeSchema = z.enum(['wsl', 'ssh', 'desktop'])

const DockerConnectionSchema = z.object({
  id: z.string().min(1),
  name: z.string(),
  type: ConnectionTypeSchema,
  wslDistro: z.string().optional(),
  sshHost: z.string().optional(),
  sshPort: z.number().int().min(1).max(65535).optional(),
  sshUser: z.string().optional(),
  sshPassword: z.string().optional(),
  useSudo: z.boolean().optional()
})

const RegistrySchema = z.object({
  id: z.string().min(1),
  name: z.string(),
  url: z.string(),
  username: z.string().optional(),
  password: z.string().optional(),
  isDefault: z.boolean().optional()
})

export const SettingsFileSchema = z.object({
  theme: ThemeSchema.optional(),
  closeToTray: z.boolean().optional(),
  refreshInterval: z.number().int().min(1).max(60).optional(),
  visibleMenus: z.array(z.string()).optional(),
  connections: z.array(DockerConnectionSchema).optional(),
  activeConnectionId: z.string().optional(),
  registries: z.array(RegistrySchema).optional(),
  currentRegistryId: z.string().optional(),
  // 老版本扁平字段（迁移路径用）
  connectionMode: ConnectionTypeSchema.optional(),
  wslDistro: z.string().optional(),
  sshHost: z.string().optional(),
  sshPort: z.number().int().min(1).max(65535).optional(),
  sshUser: z.string().optional(),
  sshPassword: z.string().optional()
})

export type SettingsFile = z.infer<typeof SettingsFileSchema>

/**
 * 把单个字段值通过对应 schema 解析；
 * - 命中：返回解析后的值；
 * - 不命中（坏数据 / 类型不符）：返回 fallback、并 console.warn 给调试线索。
 *
 * 注意我们没有对整个文件一次性 parse，而是逐字段：因为 Tauri Store 没有
 * 「读取所有键」的便捷接口，只能逐 key 取，逐 key 校验更贴合现有 loadSettings 形态。
 */
export function safeParseField<T extends z.ZodTypeAny>(
  schema: T,
  value: unknown,
  fallback: z.infer<T>,
  key: string
): z.infer<T> {
  const result = schema.safeParse(value)
  if (result.success) {
    return result.data
  }
  console.warn(`settings.json 字段 ${key} 格式非法，已回退默认值:`, result.error.issues)
  return fallback
}

export const settingsFieldSchemas = {
  theme: ThemeSchema,
  closeToTray: z.boolean(),
  refreshInterval: z.number().int().min(1).max(60),
  visibleMenus: z.array(z.string()),
  connections: z.array(DockerConnectionSchema),
  activeConnectionId: z.string(),
  registries: z.array(RegistrySchema),
  currentRegistryId: z.string(),
  // legacy
  connectionMode: ConnectionTypeSchema,
  wslDistro: z.string(),
  sshHost: z.string(),
  sshPort: z.number().int().min(1).max(65535),
  sshUser: z.string(),
  sshPassword: z.string()
}
