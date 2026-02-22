use serde::Deserialize;
use tracing::{info, debug, warn};
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::models::{OneBotEvent, MessageElement};

/// 消息解析器
pub struct MessageParser;

/// 解析后的消息内容，包含原始消息和转发消息内容
#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub raw_text: String,
    pub reply_content: Option<String>,
    pub forward_content: Option<String>,
    pub has_forward: bool,
    pub has_reply: bool,
    pub image_result: crate::models::ImageProcessingResult,
}

/// 从OneBot事件中解析消息内容
pub fn parse_message(event: &OneBotEvent) -> ParsedMessage {
    let mut parsed = ParsedMessage {
        raw_text: event.raw_message.clone().unwrap_or_default(),
        reply_content: None,
        forward_content: None,
        has_forward: false,
        has_reply: false,
        image_result: crate::models::ImageProcessingResult::default(),
    };

    // 首先尝试从raw字段解析详细消息结构
    if let Some(raw_value) = &event.raw {
        info!("开始解析raw字段数据: {:#?}", raw_value);
        if let Ok(raw_data) = serde_json::from_value::<RawMessageData>(raw_value.clone()) {
            info!("成功解析RawMessageData结构");
            parse_raw_message_data(&raw_data, &mut parsed);
        } else {
            info!("无法解析为RawMessageData结构，尝试直接查找multiForwardMsgElement");
            // 如果标准解析失败，尝试直接查找multiForwardMsgElement
            if let Some(records) = raw_value.get("records") {
                if let Some(records_array) = records.as_array() {
                    for record in records_array {
                        if let Some(elements) = record.get("elements") {
                            if let Some(elements_array) = elements.as_array() {
                                for element in elements_array {
                                    if let Some(multi_forward) = element.get("multiForwardMsgElement") {
                                        info!("找到multiForwardMsgElement: {:#?}", multi_forward);
                                        if let Some(xml_content) = multi_forward.get("xmlContent").and_then(|v| v.as_str()) {
                                            info!("找到XML内容，长度: {} 字符", xml_content.len());
                                            if let Ok(extracted_text) = extract_forward_content(xml_content) {
                                                info!("成功提取转发内容: {} 字符", extracted_text.len());
                                                parsed.forward_content = Some(extracted_text);
                                                parsed.has_forward = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 然后从message字段解析消息元素
    if let Some(message_elements) = &event.message {
        parse_message_elements(message_elements, &mut parsed);
    }

    // 最后从raw_message中提取CQ码信息
    let raw_text = parsed.raw_text.clone();
    extract_cq_codes(&raw_text, &mut parsed);

    info!("消息解析结果: has_reply={}, has_forward={}, reply_content_len={}, forward_content_len={}", 
          parsed.has_reply, parsed.has_forward, 
          parsed.reply_content.as_ref().map(|s| s.len()).unwrap_or(0),
          parsed.forward_content.as_ref().map(|s| s.len()).unwrap_or(0));

    parsed
}

/// 原始消息数据结构
#[derive(Deserialize, Debug)]
struct RawMessageData {
    elements: Option<Vec<MessageElement>>,
    records: Option<Vec<MessageRecord>>,
}

/// 消息记录结构（用于转发消息）
#[derive(Deserialize, Debug)]
struct MessageRecord {
    elements: Option<Vec<MessageElement>>,
}

/// 从原始消息数据解析
fn parse_raw_message_data(raw_data: &RawMessageData, parsed: &mut ParsedMessage) {
    debug!("开始解析原始消息数据，元素数量: {:?}, 记录数量: {:?}", 
           raw_data.elements.as_ref().map(|v| v.len()), 
           raw_data.records.as_ref().map(|v| v.len()));

    // 解析元素中的转发消息
    if let Some(elements) = &raw_data.elements {
        for element in elements {
            debug!("检查元素类型: {}", element.r#type);
            if element.r#type == "multiForwardMsg" {
                if let Ok(forward_data) = serde_json::from_value::<ForwardMessageData>(element.data.clone()) {
                    if let Some(xml_content) = forward_data.xml_content {
                        if let Ok(extracted_text) = extract_forward_content(&xml_content) {
                            debug!("从raw数据解析到转发消息内容: {} 字符", extracted_text.len());
                            parsed.forward_content = Some(extracted_text);
                            parsed.has_forward = true;
                        }
                    }
                }
            }
        }
    }

    // 解析记录中的消息内容 - 这是主要的转发消息来源
    if let Some(records) = &raw_data.records {
        debug!("开始解析记录，记录数量: {}", records.len());
        let mut record_texts = Vec::new();
        for (i, record) in records.iter().enumerate() {
            debug!("解析第 {} 条记录", i);
            if let Some(elements) = &record.elements {
                debug!("记录中的元素数量: {}", elements.len());
                for (j, element) in elements.iter().enumerate() {
                    debug!("检查第 {} 个元素类型: {}", j, element.r#type);
                    // 检查是否有 multiForwardMsgElement
                    if let Ok(multi_forward_data) = serde_json::from_value::<MultiForwardElementData>(element.data.clone()) {
                        debug!("成功解析到 MultiForwardElementData");
                        if let Some(xml_content) = multi_forward_data.multi_forward_msg_element.xml_content {
                            debug!("XML内容长度: {} 字符", xml_content.len());
                            if let Ok(extracted_text) = extract_forward_content(&xml_content) {
                                debug!("从记录中的转发消息解析到内容: {} 字符", extracted_text.len());
                                record_texts.push(extracted_text);
                            }
                        } else {
                            debug!("没有找到 XML 内容");
                        }
                    } else {
                        debug!("无法解析为 MultiForwardElementData");
                    }
                }
            }
        }
        if !record_texts.is_empty() {
            parsed.forward_content = Some(record_texts.join("\n---\n"));
            parsed.has_forward = true;
        }
    }
}

/// 转发消息元素数据结构（用于记录中的转发消息）
#[derive(Deserialize, Debug)]
struct MultiForwardElementData {
    pub multi_forward_msg_element: MultiForwardMsgElement,
}

#[derive(Deserialize, Debug)]
struct MultiForwardMsgElement {
    pub xml_content: Option<String>,
    pub res_id: Option<String>,
    pub file_name: Option<String>,
}

/// 转发消息数据结构
#[derive(Deserialize, Debug)]
struct ForwardMessageData {
    pub xml_content: Option<String>,
    pub res_id: Option<String>,
    pub file_name: Option<String>,
}

/// 从消息元素解析
fn parse_message_elements(elements: &[MessageElement], parsed: &mut ParsedMessage) {
    for element in elements {
        match element.r#type.as_str() {
            "reply" => {
                if let Ok(reply_data) = serde_json::from_value::<ReplyData>(element.data.clone()) {
                    parsed.has_reply = true;
                    // 这里可以进一步获取被回复消息的内容
                    debug!("检测到回复消息，ID: {}", reply_data.id);
                }
            }
            "forward" | "multiForwardMsg" => {
                if let Ok(forward_data) = serde_json::from_value::<ForwardMessageData>(element.data.clone()) {
                    if let Some(xml_content) = forward_data.xml_content {
                        if let Ok(extracted_text) = extract_forward_content(&xml_content) {
                            debug!("从消息元素解析到转发消息内容: {} 字符", extracted_text.len());
                            parsed.forward_content = Some(extracted_text);
                            parsed.has_forward = true;
                        }
                    }
                }
            }
            "image" => {
                if let Ok(image_data) = serde_json::from_value::<crate::models::ImageElement>(element.data.clone()) {
                    debug!("检测到图片消息: file={}, url={:?}", image_data.file, image_data.url);
                    parsed.image_result.has_image = true;
                    parsed.image_result.image_urls.push(image_data.url.unwrap_or_default());
                    
                    // 添加图片描述占位符，实际使用时可以调用图片识别服务
                    let description = format!("[图片: {}]", image_data.file);
                    parsed.image_result.image_descriptions.push(description);
                    parsed.image_result.processing_status = crate::models::ImageProcessingStatus::NotProcessed;
                }
            }
            _ => {}
        }
    }
}

/// 回复数据结构
#[derive(Deserialize, Debug)]
struct ReplyData {
    pub id: String,
}

/// 从CQ码中提取信息
fn extract_cq_codes(raw_text: &str, parsed: &mut ParsedMessage) {
    // 提取回复CQ码
    if let Some(reply_start) = raw_text.find("[CQ:reply,id=") {
        if let Some(reply_end) = raw_text[reply_start..].find(']') {
            parsed.has_reply = true;
            debug!("从CQ码检测到回复消息");
        }
    }

    // 提取转发CQ码
    if raw_text.contains("[CQ:forward,") || raw_text.contains("[聊天记录]") {
        parsed.has_forward = true;
        debug!("从CQ码检测到转发消息");
    }
}

/// 从XML内容中提取转发消息的文本内容
pub fn extract_forward_content(xml_content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(xml_content);
    reader.trim_text(true);

    let mut texts = Vec::new();
    let mut buf = Vec::new();
    let mut current_text = String::new();
    let mut in_title = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"title" {
                    in_title = true;
                    current_text.clear();
                }
            }
            Ok(Event::Text(e)) => {
                if in_title {
                    current_text.push_str(&e.unescape()?);
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"title" && in_title {
                    in_title = false;
                    let text = current_text.trim();
                    // 过滤掉标题行和空内容，但保留其他所有内容
                    if !text.contains("群聊的聊天记录") && !text.is_empty() {
                        texts.push(text.to_string());
                    }
                    current_text.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("XML解析错误: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    // 也尝试直接提取所有文本内容作为备选
    if texts.is_empty() {
        let all_text = regex::Regex::new(r">([^<]+)<")
            .unwrap()
            .captures_iter(xml_content)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().trim())
            .filter(|s| !s.is_empty() && !s.contains("群聊的聊天记录"))
            .collect::<Vec<_>>();
        
        if !all_text.is_empty() {
            texts = all_text.into_iter().map(|s| s.to_string()).collect();
        }
    }

    if texts.is_empty() {
        return Err("未找到有效的转发消息内容".into());
    }

    let result = texts.join("\n");
    info!("成功提取转发消息内容: {} 字符", result.len());
    Ok(result)
}

/// 获取用于知识库搜索的完整内容
pub fn get_searchable_content(parsed: &ParsedMessage) -> String {
    let mut content = parsed.raw_text.clone();
    
    // 移除CQ码，保留纯文本
    content = remove_cq_codes(&content);
    
    // 添加回复内容
    if let Some(reply_content) = &parsed.reply_content {
        content.push_str("\n回复内容: ");
        content.push_str(reply_content);
    }
    
    // 添加转发内容
    if let Some(forward_content) = &parsed.forward_content {
        content.push_str("\n转发内容: ");
        content.push_str(forward_content);
    }
    
    // 添加图片描述
    if parsed.image_result.has_image && !parsed.image_result.image_descriptions.is_empty() {
        content.push_str("\n图片内容: ");
        content.push_str(&parsed.image_result.image_descriptions.join(", "));
    }
    
    content.trim().to_string()
}

/// 移除CQ码，保留纯文本
pub fn remove_cq_codes(text: &str) -> String {
    let mut result = text.to_string();
    
    // 使用正则表达式移除所有CQ码，包括回复、@、转发等
    let cq_regex = regex::Regex::new(r"\[CQ:[^\]]+\]").unwrap();
    result = cq_regex.replace_all(&result, "").to_string();
    
    // 额外清理转发消息标记
    result = result.replace("[聊天记录]", "");
    result = result.replace("[转发消息]", "");
    
    // 清理多余的空白字符
    result = result.split_whitespace().collect::<Vec<&str>>().join(" ");
    
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_forward_content() {
        let xml = r#"<?xml version='1.0' encoding='UTF-8' standalone="yes"?> 
        <msg serviceID="35" templateID="1" action="viewMultiMsg" brief="[聊天记录]" m_fileName="1c5bd419-e139-43dd-ba30-299828130476" m_resid="lr6jltTHUIVyYdrxK8SZNnjBVOL2Hkln+vY60EPdMWbcOPzS01hIxflCX/p9VIAC" tSum="6" flag="3">
            <item layout="1">
                <title color="black" size="34">群聊的聊天记录</title>
                <title color="gray" size="26">UserA: 这是一条测试消息内容</title>
                <title color="gray" size="26">UserB: 这是另一条测试消息</title>
                <title color="gray" size="26">UserC: https://example.com/test</title>
                <title color="gray" size="26">UserD: [图片]</title>
                <hr></hr>
                <summary color="gray">查看6条转发消息</summary>
            </item>
            <source name="群聊的聊天记录"></source>
        </msg>"#;

        let result = extract_forward_content(xml).unwrap();
        // 验证提取的内容不为空且格式正确
        assert!(!result.is_empty());
        assert!(result.contains(":")); // 应该包含用户名和内容的分隔符
        assert!(!result.contains("群聊的聊天记录")); // 不应该包含标题行
    }

    #[test]
    fn test_remove_cq_codes() {
        let text = "[CQ:reply,id=1180219045][CQ:at,qq=1234567890] 你知道里面是什么东西吗?";
        let result = remove_cq_codes(text);
        assert_eq!(result, "你知道里面是什么东西吗?");
    }
}
