/**
 * 格式化字节数为人类可读的字符串
 */
export const formatBytes = (bytes: number, decimals = 2) => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

/**
 * 格式化时间戳为 "xxx 前"
 */
export const timeAgo = (val: number | string) => {
  if (!val) return ''
  const timestamp = typeof val === 'number' ? val : Math.floor(new Date(val).getTime() / 1000)
  const seconds = Math.floor(Date.now() / 1000 - timestamp)
  let interval = seconds / 31536000
  if (interval > 1) return Math.floor(interval) + ' 年前'
  interval = seconds / 2592000
  if (interval > 1) return Math.floor(interval) + ' 个月前'
  interval = seconds / 86400
  if (interval > 1) return Math.floor(interval) + ' 天前'
  interval = seconds / 3600
  if (interval > 1) return Math.floor(interval) + ' 小时前'
  interval = seconds / 60
  if (interval > 1) return Math.floor(interval) + ' 分钟前'
  return Math.floor(seconds) + ' 秒前'
}
