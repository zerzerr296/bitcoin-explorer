import React, { useState, useEffect } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import axios from 'axios';

interface DataPoint {
    height: number;
    transactions: number;
    price: number;
    time: string;  // 确保 time 字段是字符串类型
}

function App() {
    const [data, setData] = useState<DataPoint[]>([]);
    const apiUrl = 'http://104.154.105.117:3030'; 
    const wsUrl = 'ws://104.154.105.117:3030/ws';

    // 获取初始数据
    useEffect(() => {
        async function fetchInitialData() {
            try {
                const response = await axios.get(`${apiUrl}/latest_blocks`);
                // 不需要再映射 time 字段，直接将整个 item 对象传递给 setData
                const formattedData = response.data.reverse();
                console.log("Initial Data:", formattedData);  // 输出到控制台，检查数据格式
                setData(formattedData);
            } catch (error) {
                console.error('Failed to fetch initial data:', error);
            }
        }
    
        fetchInitialData();
    }, []);    

    // WebSocket 连接
    useEffect(() => {
        const socket = new WebSocket(wsUrl);

        socket.onmessage = (event) => {
            console.log("WebSocket message received: ", event.data);
            try {
                const parsedData = JSON.parse(event.data);
                // 直接使用 WebSocket 返回的时间字段
                console.log("Parsed Data:", parsedData);  // 输出接收到的 WebSocket 数据
                const newDataPoint: DataPoint = {
                    ...parsedData,
                    time: parsedData.timestamp, // 直接从 WebSocket 数据中获取 time 字段
                };

                setData((prevData) => {
                    const updatedData = [...prevData, newDataPoint];
                    // 保留最新的10条数据
                    return updatedData.slice(-10);
                });
            } catch (error) {
                console.error("Failed to parse WebSocket message:", error);
            }
        };

        socket.onopen = () => {
            console.log('WebSocket connection opened');
        };

        socket.onerror = (error) => {
            console.error('WebSocket error: ', error);
        };

        socket.onclose = (event) => {
            console.error('WebSocket connection closed:', event);
        };

        return () => {
            socket.close();
        };
    }, []);

    return (
        <div className="App">
            <h1>Bitcoin Explorer - Real-time Data Visualization</h1>

            {data.length > 0 ? (
                <div className="chart-container">
                    <ResponsiveContainer width="90%" height={700}>
                        <LineChart data={data}>
                            <CartesianGrid strokeDasharray="3 3" />
                            <XAxis 
                                dataKey="time"  // 使用 time 字段
                                angle={45} // 设置 X 轴刻度的倾斜角度
                                textAnchor="start" // 设置文本起始位置
                            />
                            <YAxis />
                            <Tooltip />
                            <Legend />
                            <Line type="monotone" dataKey="height" stroke="#8884d8" activeDot={{ r: 8 }} />
                            <Line type="monotone" dataKey="transactions" stroke="#82ca9d" />
                            <Line type="monotone" dataKey="price" stroke="#ff7300" />
                        </LineChart>
                    </ResponsiveContainer>
                </div>
            ) : (
                <p>Waiting for data...</p>
            )}
        </div>
    );
}

export default App;
