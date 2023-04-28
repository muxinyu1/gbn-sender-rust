import json
import subprocess
import time
import copy
import matplotlib
matplotlib.use('Agg')  # 设置非GUI后端
import matplotlib.pyplot as plt


# 读取配置文件
with open('config.json', 'r') as f:
    config = json.load(f)

# 可变参数列表
data_sizes = [128, 256, 512, 1024, 2048, 4096]
data_time = []
lost_rates = [0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90]
lostrate_time = []
timeouts = [250, 500, 750, 1000, 1250, 1500, 1750, 2000, 2250, 2500, 2750, 3000, 3250, 3500, 3750, 4000]
timeout_time = []


# data_size与执行时间的关系
cfg_clone = copy.deepcopy(config)
for size in data_sizes:
    cfg_clone['DataSize'] = size
    with open('config.json', 'w') as f:
        json.dump(cfg_clone, f)
        pass
    start_time = time.time()
    subprocess.run(['cargo', 'run', '--release'], check=True)
    end_time = time.time()
    data_time.append(end_time - start_time)
    pass

# lostrate
with open('config.json', 'w') as f:
        json.dump(config, f)
        pass
cfg_clone = copy.deepcopy(config)
for rate in lost_rates:
    cfg_clone['LostRate'] = rate
    with open('config.json', 'w') as f:
        json.dump(cfg_clone, f)
        pass
    start_time = time.time()
    subprocess.run(['cargo', 'run', '--release'], check=True)
    end_time = time.time()
    lostrate_time.append(end_time - start_time)
    pass

# timeout
# lostrate
with open('config.json', 'w') as f:
        json.dump(config, f)
        pass
cfg_clone = copy.deepcopy(config)
for timeout in timeouts:
    cfg_clone['Timeout'] = timeout
    with open('config.json', 'w') as f:
        json.dump(cfg_clone, f)
        pass
    start_time = time.time()
    subprocess.run(['cargo', 'run', '--release'], check=True)
    end_time = time.time()
    timeout_time.append(end_time - start_time)
    pass

# 恢复原json文件
with open('config.json', 'w') as f:
        json.dump(config, f)
        pass

# 画三张图

plt.plot(data_sizes, data_time)
plt.xlabel('DataSize')
plt.ylabel('Time (seconds)')
plt.savefig('datasize.png')

plt.clf()

plt.plot(lost_rates, lostrate_time)
plt.xlabel('LostRate')
plt.ylabel('Time (seconds)')
plt.savefig('lostrate.png')

plt.clf()

plt.plot(timeouts, timeout_time)
plt.xlabel('Timeout')
plt.ylabel('Time (seconds)')
plt.savefig('timeout.png')
