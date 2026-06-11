-- v005: 双 LLM 配置 + Thinking 模式 + 降级链
-- 支持 Quick-Think / Deep-Think 双模型分工
-- 支持 DeepSeek Thinking 模式开关
-- 支持多供应商降级链排序

-- 添加 thinking_enabled 列（默认关闭，向后兼容）
ALTER TABLE llm_config ADD COLUMN thinking_enabled INTEGER NOT NULL DEFAULT 0;

-- 添加 config_tier 列：'quick' 或 'deep'（默认 quick，向后兼容）
ALTER TABLE llm_config ADD COLUMN config_tier TEXT NOT NULL DEFAULT 'quick';

-- 添加 fallback_order 列：降级链中的优先级顺序（0 = 主选）
ALTER TABLE llm_config ADD COLUMN fallback_order INTEGER NOT NULL DEFAULT 0;
