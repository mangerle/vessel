// scripts/extract-changelog.cjs
const fs = require('fs');
const path = require('path');

const changelogPath = path.join(__dirname, '../CHANGELOG.md');
if (!fs.existsSync(changelogPath)) {
  console.log('Vessel 自动加签构建版本');
  process.exit(0);
}

const content = fs.readFileSync(changelogPath, 'utf8').replace(/\r\n/g, '\n');

// 匹配第一个版本标题（## 或 ### 开头且包含 [X.Y.Z] 格式版本号）到下一个版本标题（或文件结尾）之间的所有日志
const regex = /(?:##|###)\s+\[\d+\.\d+\.\d+\].*?\n([\s\S]*?)(?=(?:##|###)\s+\[\d+\.\d+\.\d+\]|$)/;
const match = content.match(regex);

if (match && match[1]) {
  console.log(match[1].trim());
} else {
  console.log('Vessel 自动加签构建版本');
}
