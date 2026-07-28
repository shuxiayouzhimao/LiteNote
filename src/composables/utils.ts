// 工具函数

/** 将时间戳转换为相对时间显示 */
export function formatTime(timeStr: string): string {
  if (!timeStr) return "";
  const dt = new Date(timeStr.replace(" ", "T"));
  if (isNaN(dt.getTime())) return timeStr;

  const now = new Date();
  const diffMs = now.getTime() - dt.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) return "刚刚";
  if (diffMin < 60) return `${diffMin}分钟前`;
  if (diffHour < 24 && now.getDate() === dt.getDate()) return `${diffHour}小时前`;
  if (diffDay === 1) return "昨天";
  if (diffDay < 7) return `${diffDay}天前`;

  // 更早：显示日期
  const y = dt.getFullYear();
  const m = String(dt.getMonth() + 1).padStart(2, "0");
  const d = String(dt.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

/** 截断文本用于预览 */
export function truncate(text: string, maxLen = 20): string {
  if (!text) return "";
  const clean = text.replace(/[\r\n]+/g, " ").trim();
  if (clean.length <= maxLen) return clean;
  return clean.slice(0, maxLen) + "...";
}
