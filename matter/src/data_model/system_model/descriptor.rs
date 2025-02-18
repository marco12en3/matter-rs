/*
 *
 *    Copyright (c) 2020-2022 Project CHIP Authors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use num_derive::FromPrimitive;

use crate::data_model::core::DataModel;
use crate::data_model::objects::*;
use crate::error::*;
use crate::interaction_model::messages::GenericPath;
use crate::tlv::{TLVWriter, TagType, ToTLV};
use log::error;

pub const ID: u32 = 0x001D;

#[derive(FromPrimitive)]
#[allow(clippy::enum_variant_names)]
pub enum Attributes {
    DeviceTypeList = 0,
    ServerList = 1,
    ClientList = 2,
    PartsList = 3,
}

pub struct DescriptorCluster {
    base: Cluster,
    endpoint_id: EndptId,
    data_model: DataModel,
}

impl DescriptorCluster {
    pub fn new(endpoint_id: EndptId, data_model: DataModel) -> Result<Box<Self>, Error> {
        let mut c = Box::new(DescriptorCluster {
            endpoint_id,
            data_model,
            base: Cluster::new(ID)?,
        });
        let attrs = [
            Attribute::new(
                Attributes::DeviceTypeList as u16,
                AttrValue::Custom,
                Access::RV,
                Quality::NONE,
            ),
            Attribute::new(
                Attributes::ServerList as u16,
                AttrValue::Custom,
                Access::RV,
                Quality::NONE,
            ),
            Attribute::new(
                Attributes::PartsList as u16,
                AttrValue::Custom,
                Access::RV,
                Quality::NONE,
            ),
            Attribute::new(
                Attributes::ClientList as u16,
                AttrValue::Custom,
                Access::RV,
                Quality::NONE,
            ),
        ];
        c.base.add_attributes(&attrs[..])?;
        Ok(c)
    }

    fn encode_devtype_list(&self, tag: TagType, tw: &mut TLVWriter) {
        let path = GenericPath {
            endpoint: Some(self.endpoint_id),
            cluster: None,
            leaf: None,
        };
        let _ = tw.start_array(tag);
        let dm = self.data_model.node.read().unwrap();
        let _ = dm.for_each_endpoint(&path, |_, e| {
            let dev_type = e.get_dev_type();
            let _ = dev_type.to_tlv(tw, TagType::Anonymous);
            Ok(())
        });
        let _ = tw.end_container();
    }

    fn encode_server_list(&self, tag: TagType, tw: &mut TLVWriter) {
        let path = GenericPath {
            endpoint: Some(self.endpoint_id),
            cluster: None,
            leaf: None,
        };
        let _ = tw.start_array(tag);
        let dm = self.data_model.node.read().unwrap();
        let _ = dm.for_each_cluster(&path, |_current_path, c| {
            let _ = tw.u32(TagType::Anonymous, c.base().id());
            Ok(())
        });
        let _ = tw.end_container();
    }

    fn encode_parts_list(&self, tag: TagType, tw: &mut TLVWriter) {
        let path = GenericPath {
            endpoint: None,
            cluster: None,
            leaf: None,
        };
        let _ = tw.start_array(tag);
        if self.endpoint_id == 0 {
            // TODO: If endpoint is another than 0, need to figure out what to do
            let dm = self.data_model.node.read().unwrap();
            let _ = dm.for_each_endpoint(&path, |current_path, _| {
                if let Some(endpoint_id) = current_path.endpoint {
                    if endpoint_id != 0 {
                        let _ = tw.u16(TagType::Anonymous, endpoint_id);
                    }
                }
                Ok(())
            });
        }
        let _ = tw.end_container();
    }

    fn encode_client_list(&self, tag: TagType, tw: &mut TLVWriter) {
        // No Clients supported
        let _ = tw.start_array(tag);
        let _ = tw.end_container();
    }
}

impl ClusterType for DescriptorCluster {
    fn base(&self) -> &Cluster {
        &self.base
    }
    fn base_mut(&mut self) -> &mut Cluster {
        &mut self.base
    }

    fn read_custom_attribute(&self, encoder: &mut dyn Encoder, attr: &AttrDetails) {
        match num::FromPrimitive::from_u16(attr.attr_id) {
            Some(Attributes::DeviceTypeList) => encoder.encode(EncodeValue::Closure(&|tag, tw| {
                self.encode_devtype_list(tag, tw)
            })),
            Some(Attributes::ServerList) => encoder.encode(EncodeValue::Closure(&|tag, tw| {
                self.encode_server_list(tag, tw)
            })),
            Some(Attributes::PartsList) => encoder.encode(EncodeValue::Closure(&|tag, tw| {
                self.encode_parts_list(tag, tw)
            })),
            Some(Attributes::ClientList) => encoder.encode(EncodeValue::Closure(&|tag, tw| {
                self.encode_client_list(tag, tw)
            })),
            _ => {
                error!("Attribute not supported: this shouldn't happen");
            }
        }
    }
}
