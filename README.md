# Danmuji [![ci](https://github.com/YangchenYe323/Danmuji/actions/workflows/rust-test.yml/badge.svg)](https://github.com/YangchenYe323/Danmuji/actions/workflows/rust-test.yml)

一个使用Rust + Typescript编写的B站直播弹幕助手。

#### 后端：Axum
#### 前端：React + Tailwindcss

## 运行：

#### Docker构建：
```bash
# 获取代码
git clone git@github.com:YangchenYe323/Danmuji.git
# 进入项目目录
cd Danmuji
# 运行docker构建
docker build . -t <image_name>
# 项目监听在localhost:9000端口
docker run -p 9000:9000 <image_name>

# 访问localhost:9000
```

#### 从源码构建：
```bash
# 获取代码
git clone git@github.com:YangchenYe323/Danmuji.git
# 进入项目根目录
cd Danmuji
# 运行项目后端，监听在http://0.0.0.0:9000端口
cargo run

# 另起一个terminal, 进入前端目录
cd Danmuji/frontend
# 构建运行前端，监听在http://localhost:3000端口
npm install
npm run dev

# 访问localhost:3000
```


## 技术特点
- 实现一套对应B站API数据包的Rust类型([详情](src/client/common.rs))。使用[ts_rs](https://github.com/Aleph-Alpha/ts-rs)直接生成Typescript类型。
- 封装了面向B站直播的[websocket客户端](src/client/biliclient.rs)，使用基于broadcast的消息转发机制，在此基础上可以轻松进行插件式的功能开发（感谢姬等）以及支持多用户，多房间。

## 功能列表

- 弹幕姬
  - [x] 收集 & 转发直播间弹幕到web页面
  - [ ] 显示弹幕头像
  - [ ] 延时显示弹幕
  - [x] 显示礼物消息
  - [x] 积累一段时间内的礼物消息汇总显示
  - [ ] 礼物消息特效
  - [ ] 显示进场消息
  - [ ] 进场特效
  
- [x] 扫码登录接口
- [x] 扫码登录前端
 
- 感谢姬
  - [x] 实时感谢礼物
  - [ ] 延时汇总感谢 (Note: 低优先级。由于B站自己会把连击礼物打包成COMBO_SEND数据包，暂不确定这个功能有多重要）

- Web服务
  - [ ] 构建打包发布
