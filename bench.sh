#!/bin/bash

# MDI 性能测试运行脚本

echo "╔════════════════════════════════════════════════════════╗"
echo "║      MDI 系统 - 编译与基准测试 (Build & Benchmarks)    ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# 检查 Rust 安装
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust 未安装。请先安装 Rust: https://rustup.rs/"
    exit 1
fi

echo "✓ Rust 环境检查通过"
echo "  rustc: $(rustc --version)"
echo "  cargo: $(cargo --version)"
echo ""

# 清理旧的编译
echo "🧹 清理旧的编译文件..."
rm -rf target/.cargo-lock 2>/dev/null

# 构建项目
echo ""
echo "🔨 构建 MDI 库 (Release 模式)..."
cargo build --release --lib 2>&1 | grep -E "Compiling mdi|Finished|error" || echo "构建进行中..."

if [ ! -f target/release/libmdi.a ] && [ ! -f target/release/libmdi.so ]; then
    echo ""
    echo "⏳ 构建仍在进行中，请稍等..."
    echo "   可以在后台运行: cargo build --release"
    echo ""
fi

# 运行演示
echo ""
echo "🎬 运行 quick_bench 性能测试..."
echo "(如果编译未完成，将等待编译完成...)"
echo ""

timeout 300 cargo run --example quick_bench --release 2>&1 &
BENCH_PID=$!

# 等待基准测试完成
wait $BENCH_PID
BENCH_EXIT=$?

if [ $BENCH_EXIT -eq 0 ]; then
    echo ""
    echo "✅ 基准测试完成！"
elif [ $BENCH_EXIT -eq 124 ]; then
    echo ""
    echo "⏱️  基准测试超时 (5分钟)"
    echo "   提示: 编译可能仍在进行中。请等待后重试。"
else
    echo ""
    echo "⚠️  基准测试中断 (代码: $BENCH_EXIT)"
fi

echo ""
echo "📊 查看详细基准测试报告:"
echo "   cat BENCHMARKS.md"
echo ""
echo "📚 查看其他文档:"
echo "   cat README_ZH.md       # 中文快速开始"
echo "   cat ARCHITECTURE.md    # 系统架构"
echo "   cat IMPLEMENTATION.md  # 实现细节"
echo ""
