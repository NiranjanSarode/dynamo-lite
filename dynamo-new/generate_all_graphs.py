#!/usr/bin/env python3
"""
Comprehensive Dynamo Graph Generator

Generates all latency graphs from benchmark data:
- Basic latency histograms, percentiles, CDF
- Configuration comparison (boxplots, median, p99)
- N=20 varying R and W analysis
"""

import re
import sys
from collections import defaultdict
import matplotlib.pyplot as plt
import numpy as np

# ============================================================================
# Parsing Functions
# ============================================================================

def parse_basic_logs(log_file):
    """Parse basic Dynamo logs and extract latency measurements."""
    latencies = {'get': [], 'put': []}

    patterns = [
        re.compile(r'\[.*?\]\s+(GET|PUT)\s+key=(\S+)\s+latency=([\d.]+)ms'),
        re.compile(r'(GET|PUT)\s+key=(\S+)\s+latency=([\d.]+)ms'),
        re.compile(r'(GET|PUT)\s+key=\S+\s+latency=([\d.]+)'),
    ]

    try:
        with open(log_file, 'r') as f:
            for line in f:
                for pattern in patterns:
                    match = pattern.search(line)
                    if match:
                        op_type = match.group(1).lower()
                        latency_str = match.group(3) if len(match.groups()) >= 3 else match.group(2)
                        try:
                            latency = float(latency_str)
                            latencies[op_type].append(latency)
                            break
                        except (ValueError, IndexError):
                            continue
    except FileNotFoundError:
        print(f"Warning: Log file '{log_file}' not found")
        return None

    return latencies

def parse_config_logs(log_file):
    """Parse configuration benchmark logs."""
    config_data = defaultdict(lambda: {'get': [], 'put': []})

    pattern = re.compile(r'\[(N\d+_W\d+_R\d+)\]\s+(GET|PUT)\s+key=\S+\s+latency=([\d.]+)ms\s+N=(\d+)\s+W=(\d+)\s+R=(\d+)')

    try:
        with open(log_file, 'r') as f:
            for line in f:
                match = pattern.search(line)
                if match:
                    config_name = match.group(1)
                    op_type = match.group(2).lower()
                    latency = float(match.group(3))
                    n, w, r = int(match.group(4)), int(match.group(5)), int(match.group(6))

                    config_data[config_name][op_type].append(latency)
                    config_data[config_name]['params'] = (n, w, r)
    except FileNotFoundError:
        print(f"Warning: Log file '{log_file}' not found")
        return None

    return config_data

# ============================================================================
# Basic Latency Graphs (4 graphs)
# ============================================================================

def generate_basic_graphs(latencies, output_prefix='latency'):
    """Generate basic latency graphs: histograms, percentiles, CDF."""
    output_files = []

    # Graph 1: Read Latency Histogram
    fig1, ax1 = plt.subplots(figsize=(8, 6))
    if latencies['get']:
        ax1.hist(latencies['get'], bins=50, alpha=0.7, color='blue', edgecolor='black')
        ax1.axvline(np.percentile(latencies['get'], 50), color='green', linestyle='--',
                   linewidth=2, label=f"p50: {np.percentile(latencies['get'], 50):.2f}ms")
        ax1.axvline(np.percentile(latencies['get'], 95), color='orange', linestyle='--',
                   linewidth=2, label=f"p95: {np.percentile(latencies['get'], 95):.2f}ms")
        ax1.axvline(np.percentile(latencies['get'], 99), color='red', linestyle='--',
                   linewidth=2, label=f"p99: {np.percentile(latencies['get'], 99):.2f}ms")
        ax1.set_xlabel('Latency (ms)', fontsize=11)
        ax1.set_ylabel('Frequency', fontsize=11)
        ax1.set_title('Read Latency Distribution', fontsize=12, fontweight='bold')
        ax1.legend()
        ax1.grid(True, alpha=0.3)

    output_file1 = f'{output_prefix}_read_histogram.png'
    plt.tight_layout()
    plt.savefig(output_file1, dpi=300, bbox_inches='tight')
    plt.close(fig1)
    output_files.append(output_file1)
    print(f"✓ Saved: {output_file1}")

    # Graph 2: Write Latency Histogram
    fig2, ax2 = plt.subplots(figsize=(8, 6))
    if latencies['put']:
        ax2.hist(latencies['put'], bins=50, alpha=0.7, color='orange', edgecolor='black')
        ax2.axvline(np.percentile(latencies['put'], 50), color='green', linestyle='--',
                   linewidth=2, label=f"p50: {np.percentile(latencies['put'], 50):.2f}ms")
        ax2.axvline(np.percentile(latencies['put'], 95), color='orange', linestyle='--',
                   linewidth=2, label=f"p95: {np.percentile(latencies['put'], 95):.2f}ms")
        ax2.axvline(np.percentile(latencies['put'], 99), color='red', linestyle='--',
                   linewidth=2, label=f"p99: {np.percentile(latencies['put'], 99):.2f}ms")
        ax2.set_xlabel('Latency (ms)', fontsize=11)
        ax2.set_ylabel('Frequency', fontsize=11)
        ax2.set_title('Write Latency Distribution', fontsize=12, fontweight='bold')
        ax2.legend()
        ax2.grid(True, alpha=0.3)

    output_file2 = f'{output_prefix}_write_histogram.png'
    plt.tight_layout()
    plt.savefig(output_file2, dpi=300, bbox_inches='tight')
    plt.close(fig2)
    output_files.append(output_file2)
    print(f"✓ Saved: {output_file2}")

    # Graph 3: Percentile Comparison
    fig3, ax3 = plt.subplots(figsize=(8, 6))
    get_stats = calculate_percentiles(latencies['get'])
    put_stats = calculate_percentiles(latencies['put'])

    x = np.arange(3)
    width = 0.35

    get_vals = [get_stats['p50'], get_stats['p95'], get_stats['p99']]
    put_vals = [put_stats['p50'], put_stats['p95'], put_stats['p99']]

    bars1 = ax3.bar(x - width/2, get_vals, width, label='GET', alpha=0.8, color='blue')
    bars2 = ax3.bar(x + width/2, put_vals, width, label='PUT', alpha=0.8, color='orange')

    ax3.set_xlabel('Percentile', fontsize=11)
    ax3.set_ylabel('Latency (ms)', fontsize=11)
    ax3.set_title('Latency Percentiles Comparison', fontsize=12, fontweight='bold')
    ax3.set_xticks(x)
    ax3.set_xticklabels(['p50', 'p95', 'p99'])
    ax3.legend()
    ax3.grid(True, axis='y', alpha=0.3)

    for bars in [bars1, bars2]:
        for bar in bars:
            height = bar.get_height()
            if height > 0:
                ax3.text(bar.get_x() + bar.get_width()/2., height,
                        f'{height:.1f}', ha='center', va='bottom', fontsize=9)

    output_file3 = f'{output_prefix}_percentiles.png'
    plt.tight_layout()
    plt.savefig(output_file3, dpi=300, bbox_inches='tight')
    plt.close(fig3)
    output_files.append(output_file3)
    print(f"✓ Saved: {output_file3}")

    # Graph 4: CDF
    fig4, ax4 = plt.subplots(figsize=(8, 6))

    if latencies['get']:
        sorted_get = sorted(latencies['get'])
        y_get = np.arange(1, len(sorted_get) + 1) / len(sorted_get) * 100
        ax4.plot(sorted_get, y_get, label='GET', linewidth=2, color='blue')

    if latencies['put']:
        sorted_put = sorted(latencies['put'])
        y_put = np.arange(1, len(sorted_put) + 1) / len(sorted_put) * 100
        ax4.plot(sorted_put, y_put, label='PUT', linewidth=2, color='orange')

    ax4.set_xlabel('Latency (ms)', fontsize=11)
    ax4.set_ylabel('Cumulative Probability (%)', fontsize=11)
    ax4.set_title('Latency CDF', fontsize=12, fontweight='bold')
    ax4.legend()
    ax4.grid(True, alpha=0.3)
    ax4.set_ylim([0, 105])

    output_file4 = f'{output_prefix}_cdf.png'
    plt.tight_layout()
    plt.savefig(output_file4, dpi=300, bbox_inches='tight')
    plt.close(fig4)
    output_files.append(output_file4)
    print(f"✓ Saved: {output_file4}")

    return output_files

# ============================================================================
# Configuration Comparison Graphs (4 graphs)
# ============================================================================

def generate_config_graphs(config_data, output_prefix='latency_config'):
    """Generate configuration comparison graphs."""
    output_files = []

    sorted_configs = sorted(config_data.items(), key=lambda x: x[1]['params'][1])
    config_names = [name for name, _ in sorted_configs]
    config_labels = [f"N={data['params'][0]},W={data['params'][1]},R={data['params'][2]}"
                     for _, data in sorted_configs]

    # Graph 1: GET Latency Box Plot
    fig1, ax1 = plt.subplots(figsize=(12, 6))
    get_data = [data['get'] for _, data in sorted_configs if data['get']]
    if get_data:
        bp1 = ax1.boxplot(get_data, tick_labels=config_labels, patch_artist=True)
        for patch in bp1['boxes']:
            patch.set_facecolor('lightblue')
        ax1.set_xlabel('Configuration', fontsize=11)
        ax1.set_ylabel('Latency (ms)', fontsize=11)
        ax1.set_title('GET Latency Distribution by Config', fontsize=12, fontweight='bold')
        ax1.tick_params(axis='x', rotation=45)
        ax1.grid(True, axis='y', alpha=0.3)

    output_file1 = f'{output_prefix}_get_boxplot.png'
    plt.tight_layout()
    plt.savefig(output_file1, dpi=300, bbox_inches='tight')
    plt.close(fig1)
    output_files.append(output_file1)
    print(f"✓ Saved: {output_file1}")

    # Graph 2: PUT Latency Box Plot
    fig2, ax2 = plt.subplots(figsize=(12, 6))
    put_data = [data['put'] for _, data in sorted_configs if data['put']]
    if put_data:
        bp2 = ax2.boxplot(put_data, tick_labels=config_labels, patch_artist=True)
        for patch in bp2['boxes']:
            patch.set_facecolor('lightcoral')
        ax2.set_xlabel('Configuration', fontsize=11)
        ax2.set_ylabel('Latency (ms)', fontsize=11)
        ax2.set_title('PUT Latency Distribution by Config', fontsize=12, fontweight='bold')
        ax2.tick_params(axis='x', rotation=45)
        ax2.grid(True, axis='y', alpha=0.3)

    output_file2 = f'{output_prefix}_put_boxplot.png'
    plt.tight_layout()
    plt.savefig(output_file2, dpi=300, bbox_inches='tight')
    plt.close(fig2)
    output_files.append(output_file2)
    print(f"✓ Saved: {output_file2}")

    # Graph 3: Median Latency
    fig3, ax3 = plt.subplots(figsize=(12, 6))
    get_medians = [np.median(data['get']) if data['get'] else 0 for _, data in sorted_configs]
    put_medians = [np.median(data['put']) if data['put'] else 0 for _, data in sorted_configs]

    x = np.arange(len(config_labels))
    width = 0.35

    bars1 = ax3.bar(x - width/2, get_medians, width, label='GET (p50)', alpha=0.8, color='blue')
    bars2 = ax3.bar(x + width/2, put_medians, width, label='PUT (p50)', alpha=0.8, color='orange')

    ax3.set_xlabel('Configuration', fontsize=11)
    ax3.set_ylabel('Median Latency (ms)', fontsize=11)
    ax3.set_title('Median Latency by Configuration', fontsize=12, fontweight='bold')
    ax3.set_xticks(x)
    ax3.set_xticklabels(config_labels, rotation=45, ha='right')
    ax3.legend()
    ax3.grid(True, axis='y', alpha=0.3)

    for bars in [bars1, bars2]:
        for bar in bars:
            height = bar.get_height()
            if height > 0:
                ax3.text(bar.get_x() + bar.get_width()/2., height,
                        f'{height:.1f}', ha='center', va='bottom', fontsize=7)

    output_file3 = f'{output_prefix}_median.png'
    plt.tight_layout()
    plt.savefig(output_file3, dpi=300, bbox_inches='tight')
    plt.close(fig3)
    output_files.append(output_file3)
    print(f"✓ Saved: {output_file3}")

    # Graph 4: p99 Latency
    fig4, ax4 = plt.subplots(figsize=(12, 6))
    get_p99 = [np.percentile(data['get'], 99) if data['get'] else 0 for _, data in sorted_configs]
    put_p99 = [np.percentile(data['put'], 99) if data['put'] else 0 for _, data in sorted_configs]

    ax4.plot(config_labels, get_p99, marker='o', linewidth=2, markersize=8, label='GET (p99)', color='blue')
    ax4.plot(config_labels, put_p99, marker='s', linewidth=2, markersize=8, label='PUT (p99)', color='orange')

    ax4.set_xlabel('Configuration', fontsize=11)
    ax4.set_ylabel('p99 Latency (ms)', fontsize=11)
    ax4.set_title('p99 Latency vs Configuration', fontsize=12, fontweight='bold')
    ax4.tick_params(axis='x', rotation=45)
    ax4.legend()
    ax4.grid(True, alpha=0.3)

    output_file4 = f'{output_prefix}_p99.png'
    plt.tight_layout()
    plt.savefig(output_file4, dpi=300, bbox_inches='tight')
    plt.close(fig4)
    output_files.append(output_file4)
    print(f"✓ Saved: {output_file4}")

    return output_files

# ============================================================================
# N=20 Analysis Graphs (2 graphs)
# ============================================================================

def generate_n20_graphs(config_data, output_prefix='latency_n20'):
    """Generate N=20 varying R and W analysis graphs."""
    output_files = []

    # Extract N=20 configurations
    n20_configs = {name: data for name, data in config_data.items()
                   if data['params'][0] == 20}

    if not n20_configs:
        print("Warning: No N=20 configurations found")
        return output_files

    # Separate varying-R and varying-W configs
    varying_r = {}  # W=10, R varies
    varying_w = {}  # R=10, W varies

    for name, data in n20_configs.items():
        n, w, r = data['params']
        if w == 10:
            varying_r[r] = data
        if r == 10:
            varying_w[w] = data

    # Graph 1: N=20, W=10, Varying R
    if varying_r:
        fig1, ax1 = plt.subplots(figsize=(10, 6))

        r_values = sorted(varying_r.keys())
        get_medians = [np.median(varying_r[r]['get']) if varying_r[r]['get'] else 0 for r in r_values]
        put_medians = [np.median(varying_r[r]['put']) if varying_r[r]['put'] else 0 for r in r_values]
        get_p99 = [np.percentile(varying_r[r]['get'], 99) if varying_r[r]['get'] else 0 for r in r_values]
        put_p99 = [np.percentile(varying_r[r]['put'], 99) if varying_r[r]['put'] else 0 for r in r_values]

        x = np.arange(len(r_values))
        width = 0.35

        ax1.plot(r_values, get_medians, marker='o', linewidth=2, markersize=8,
                label='GET (p50)', color='blue', linestyle='-')
        ax1.plot(r_values, put_medians, marker='s', linewidth=2, markersize=8,
                label='PUT (p50)', color='orange', linestyle='-')
        ax1.plot(r_values, get_p99, marker='o', linewidth=2, markersize=6,
                label='GET (p99)', color='blue', linestyle='--', alpha=0.6)
        ax1.plot(r_values, put_p99, marker='s', linewidth=2, markersize=6,
                label='PUT (p99)', color='orange', linestyle='--', alpha=0.6)

        ax1.set_xlabel('Read Quorum (R)', fontsize=11)
        ax1.set_ylabel('Latency (ms)', fontsize=11)
        ax1.set_title('N=20, W=10: Latency vs Read Quorum (R)', fontsize=12, fontweight='bold')
        ax1.legend()
        ax1.grid(True, alpha=0.3)
        ax1.set_xticks(r_values)

        output_file1 = f'{output_prefix}_varying_r.png'
        plt.tight_layout()
        plt.savefig(output_file1, dpi=300, bbox_inches='tight')
        plt.close(fig1)
        output_files.append(output_file1)
        print(f"✓ Saved: {output_file1}")

    # Graph 2: N=20, R=10, Varying W
    if varying_w:
        fig2, ax2 = plt.subplots(figsize=(10, 6))

        w_values = sorted(varying_w.keys())
        get_medians = [np.median(varying_w[w]['get']) if varying_w[w]['get'] else 0 for w in w_values]
        put_medians = [np.median(varying_w[w]['put']) if varying_w[w]['put'] else 0 for w in w_values]
        get_p99 = [np.percentile(varying_w[w]['get'], 99) if varying_w[w]['get'] else 0 for w in w_values]
        put_p99 = [np.percentile(varying_w[w]['put'], 99) if varying_w[w]['put'] else 0 for w in w_values]

        ax2.plot(w_values, get_medians, marker='o', linewidth=2, markersize=8,
                label='GET (p50)', color='blue', linestyle='-')
        ax2.plot(w_values, put_medians, marker='s', linewidth=2, markersize=8,
                label='PUT (p50)', color='orange', linestyle='-')
        ax2.plot(w_values, get_p99, marker='o', linewidth=2, markersize=6,
                label='GET (p99)', color='blue', linestyle='--', alpha=0.6)
        ax2.plot(w_values, put_p99, marker='s', linewidth=2, markersize=6,
                label='PUT (p99)', color='orange', linestyle='--', alpha=0.6)

        ax2.set_xlabel('Write Quorum (W)', fontsize=11)
        ax2.set_ylabel('Latency (ms)', fontsize=11)
        ax2.set_title('N=20, R=10: Latency vs Write Quorum (W)', fontsize=12, fontweight='bold')
        ax2.legend()
        ax2.grid(True, alpha=0.3)
        ax2.set_xticks(w_values)

        output_file2 = f'{output_prefix}_varying_w.png'
        plt.tight_layout()
        plt.savefig(output_file2, dpi=300, bbox_inches='tight')
        plt.close(fig2)
        output_files.append(output_file2)
        print(f"✓ Saved: {output_file2}")

    return output_files

# ============================================================================
# Helper Functions
# ============================================================================

def calculate_percentiles(data):
    """Calculate percentile statistics."""
    if not data:
        return {'p50': 0, 'p95': 0, 'p99': 0, 'mean': 0, 'min': 0, 'max': 0}

    sorted_data = sorted(data)
    return {
        'p50': np.percentile(sorted_data, 50),
        'p95': np.percentile(sorted_data, 95),
        'p99': np.percentile(sorted_data, 99),
        'mean': np.mean(sorted_data),
        'min': min(sorted_data),
        'max': max(sorted_data)
    }

# ============================================================================
# Main
# ============================================================================

def main():
    """Main entry point - generate all graphs."""
    print("="*60)
    print("Dynamo Comprehensive Graph Generator")
    print("="*60)
    print()

    total_graphs = 0

    # 1. Basic Latency Graphs
    print("1. Generating Basic Latency Graphs...")
    latencies = parse_basic_logs('latency.log')
    if latencies and (latencies['get'] or latencies['put']):
        files = generate_basic_graphs(latencies)
        total_graphs += len(files)
        print(f"   Generated {len(files)} basic graphs")
    else:
        print("   Skipped: No basic latency data found")
    print()

    # 2. Configuration Comparison Graphs
    print("2. Generating Configuration Comparison Graphs...")
    config_data = parse_config_logs('latency_configs.log')
    if config_data:
        files = generate_config_graphs(config_data)
        total_graphs += len(files)
        print(f"   Generated {len(files)} config comparison graphs")

        # 3. N=20 Analysis Graphs
        print()
        print("3. Generating N=20 Analysis Graphs...")
        files = generate_n20_graphs(config_data)
        total_graphs += len(files)
        print(f"   Generated {len(files)} N=20 analysis graphs")
    else:
        print("   Skipped: No configuration data found")

    print()
    print("="*60)
    print(f"✓ Complete! Generated {total_graphs} total graphs")
    print("="*60)

if __name__ == '__main__':
    main()
