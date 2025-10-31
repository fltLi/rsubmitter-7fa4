"""
rsubmitter 构建脚本
使用方法: python build.py <输出目录>
"""

import sys
import shutil
import subprocess
import argparse
from pathlib import Path

def build_wasm(output_dir: str):
    """ 构建 WASM 模块到指定输出目录的 wasm 子目录中 """
    # 将输出目录转换为绝对路径
    output_path = Path(output_dir).resolve()
    wasm_output = output_path / "wasm"
    wasm_output.mkdir(parents=True, exist_ok=True)
    
    print(f"构建 WASM 到 {wasm_output}...")
    
    result = subprocess.run(
        ["wasm-pack", "build", "--release", "--target", "web", "--out-dir", str(wasm_output)],
        cwd="runtime"
    )
    
    if result.returncode != 0:
        raise RuntimeError("WASM 构建失败")
    
    print("WASM 构建完成")

def copy_extension(output_dir: str):
    """ 复制扩展文件到指定输出目录 """
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)
    
    print(f"复制扩展文件到 {output_path}...")
    
    # 要复制的文件列表
    files = [
        "manifest.json",
        "popup.html",
        "popup.js",
        "popup.css",
        "core.js",
        "ui.js",
    ]
    
    # 复制每个文件
    for file in files:
        src = Path("extension") / file
        dst = output_path / file
        
        if not src.exists():
            print(f"警告: 找不到源文件 {src}")
            continue
            
        shutil.copy2(src, dst)
        print(f"  已复制: {file}")
    
    print("扩展文件复制完成")

def main():
    parser = argparse.ArgumentParser(description="构建 Chrome 插件")
    parser.add_argument("output_dir", help="输出目录路径")
    parser.add_argument("--wasm-only", action="store_true", help="仅构建 WASM")
    parser.add_argument("--copy-only", action="store_true", help="仅复制扩展文件")
    
    args = parser.parse_args()
    
    try:
        if args.wasm_only:
            build_wasm(args.output_dir)
        elif args.copy_only:
            copy_extension(args.output_dir)
        else:
            # 默认执行完整构建
            build_wasm(args.output_dir)
            copy_extension(args.output_dir)
            print(f"\n构建完成! 输出目录: {args.output_dir}")
            
            # 显示构建结果
            output_path = Path(args.output_dir)
            if output_path.exists():
                print("\n生成的文件:")
                for item in output_path.rglob("*"):
                    if item.is_file():
                        print(f"  {item.relative_to(output_path)}")
            
    except Exception as e:
        print(f"错误: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
