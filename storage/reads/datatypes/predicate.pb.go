// Code generated by protoc-gen-go. DO NOT EDIT.
// versions:
// 	protoc-gen-go v1.27.1
// 	protoc        v3.17.3
// source: predicate.proto

package datatypes

import (
	protoreflect "google.golang.org/protobuf/reflect/protoreflect"
	protoimpl "google.golang.org/protobuf/runtime/protoimpl"
	reflect "reflect"
	sync "sync"
)

const (
	// Verify that this generated code is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(20 - protoimpl.MinVersion)
	// Verify that runtime/protoimpl is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(protoimpl.MaxVersion - 20)
)

type Node_Type int32

const (
	Node_TypeLogicalExpression    Node_Type = 0
	Node_TypeComparisonExpression Node_Type = 1
	Node_TypeParenExpression      Node_Type = 2
	Node_TypeTagRef               Node_Type = 3
	Node_TypeLiteral              Node_Type = 4
	Node_TypeFieldRef             Node_Type = 5
)

// Enum value maps for Node_Type.
var (
	Node_Type_name = map[int32]string{
		0: "TypeLogicalExpression",
		1: "TypeComparisonExpression",
		2: "TypeParenExpression",
		3: "TypeTagRef",
		4: "TypeLiteral",
		5: "TypeFieldRef",
	}
	Node_Type_value = map[string]int32{
		"TypeLogicalExpression":    0,
		"TypeComparisonExpression": 1,
		"TypeParenExpression":      2,
		"TypeTagRef":               3,
		"TypeLiteral":              4,
		"TypeFieldRef":             5,
	}
)

func (x Node_Type) Enum() *Node_Type {
	p := new(Node_Type)
	*p = x
	return p
}

func (x Node_Type) String() string {
	return protoimpl.X.EnumStringOf(x.Descriptor(), protoreflect.EnumNumber(x))
}

func (Node_Type) Descriptor() protoreflect.EnumDescriptor {
	return file_predicate_proto_enumTypes[0].Descriptor()
}

func (Node_Type) Type() protoreflect.EnumType {
	return &file_predicate_proto_enumTypes[0]
}

func (x Node_Type) Number() protoreflect.EnumNumber {
	return protoreflect.EnumNumber(x)
}

// Deprecated: Use Node_Type.Descriptor instead.
func (Node_Type) EnumDescriptor() ([]byte, []int) {
	return file_predicate_proto_rawDescGZIP(), []int{0, 0}
}

type Node_Comparison int32

const (
	Node_ComparisonEqual        Node_Comparison = 0
	Node_ComparisonNotEqual     Node_Comparison = 1
	Node_ComparisonStartsWith   Node_Comparison = 2
	Node_ComparisonRegex        Node_Comparison = 3
	Node_ComparisonNotRegex     Node_Comparison = 4
	Node_ComparisonLess         Node_Comparison = 5
	Node_ComparisonLessEqual    Node_Comparison = 6
	Node_ComparisonGreater      Node_Comparison = 7
	Node_ComparisonGreaterEqual Node_Comparison = 8
)

// Enum value maps for Node_Comparison.
var (
	Node_Comparison_name = map[int32]string{
		0: "ComparisonEqual",
		1: "ComparisonNotEqual",
		2: "ComparisonStartsWith",
		3: "ComparisonRegex",
		4: "ComparisonNotRegex",
		5: "ComparisonLess",
		6: "ComparisonLessEqual",
		7: "ComparisonGreater",
		8: "ComparisonGreaterEqual",
	}
	Node_Comparison_value = map[string]int32{
		"ComparisonEqual":        0,
		"ComparisonNotEqual":     1,
		"ComparisonStartsWith":   2,
		"ComparisonRegex":        3,
		"ComparisonNotRegex":     4,
		"ComparisonLess":         5,
		"ComparisonLessEqual":    6,
		"ComparisonGreater":      7,
		"ComparisonGreaterEqual": 8,
	}
)

func (x Node_Comparison) Enum() *Node_Comparison {
	p := new(Node_Comparison)
	*p = x
	return p
}

func (x Node_Comparison) String() string {
	return protoimpl.X.EnumStringOf(x.Descriptor(), protoreflect.EnumNumber(x))
}

func (Node_Comparison) Descriptor() protoreflect.EnumDescriptor {
	return file_predicate_proto_enumTypes[1].Descriptor()
}

func (Node_Comparison) Type() protoreflect.EnumType {
	return &file_predicate_proto_enumTypes[1]
}

func (x Node_Comparison) Number() protoreflect.EnumNumber {
	return protoreflect.EnumNumber(x)
}

// Deprecated: Use Node_Comparison.Descriptor instead.
func (Node_Comparison) EnumDescriptor() ([]byte, []int) {
	return file_predicate_proto_rawDescGZIP(), []int{0, 1}
}

// Logical operators apply to boolean values and combine to produce a single boolean result.
type Node_Logical int32

const (
	Node_LogicalAnd Node_Logical = 0
	Node_LogicalOr  Node_Logical = 1
)

// Enum value maps for Node_Logical.
var (
	Node_Logical_name = map[int32]string{
		0: "LogicalAnd",
		1: "LogicalOr",
	}
	Node_Logical_value = map[string]int32{
		"LogicalAnd": 0,
		"LogicalOr":  1,
	}
)

func (x Node_Logical) Enum() *Node_Logical {
	p := new(Node_Logical)
	*p = x
	return p
}

func (x Node_Logical) String() string {
	return protoimpl.X.EnumStringOf(x.Descriptor(), protoreflect.EnumNumber(x))
}

func (Node_Logical) Descriptor() protoreflect.EnumDescriptor {
	return file_predicate_proto_enumTypes[2].Descriptor()
}

func (Node_Logical) Type() protoreflect.EnumType {
	return &file_predicate_proto_enumTypes[2]
}

func (x Node_Logical) Number() protoreflect.EnumNumber {
	return protoreflect.EnumNumber(x)
}

// Deprecated: Use Node_Logical.Descriptor instead.
func (Node_Logical) EnumDescriptor() ([]byte, []int) {
	return file_predicate_proto_rawDescGZIP(), []int{0, 2}
}

type Node struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	NodeType Node_Type `protobuf:"varint,1,opt,name=node_type,json=nodeType,proto3,enum=influxdata.platform.storage.Node_Type" json:"node_type,omitempty"` // [(gogoproto.customname) = "NodeType", (gogoproto.jsontag) = "nodeType"];
	Children []*Node   `protobuf:"bytes,2,rep,name=children,proto3" json:"children,omitempty"`
	// Types that are assignable to Value:
	//
	//	*Node_StringValue
	//	*Node_BooleanValue
	//	*Node_IntegerValue
	//	*Node_UnsignedValue
	//	*Node_FloatValue
	//	*Node_RegexValue
	//	*Node_TagRefValue
	//	*Node_FieldRefValue
	//	*Node_Logical_
	//	*Node_Comparison_
	Value isNode_Value `protobuf_oneof:"value"`
}

func (x *Node) Reset() {
	*x = Node{}
	if protoimpl.UnsafeEnabled {
		mi := &file_predicate_proto_msgTypes[0]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *Node) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*Node) ProtoMessage() {}

func (x *Node) ProtoReflect() protoreflect.Message {
	mi := &file_predicate_proto_msgTypes[0]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use Node.ProtoReflect.Descriptor instead.
func (*Node) Descriptor() ([]byte, []int) {
	return file_predicate_proto_rawDescGZIP(), []int{0}
}

func (x *Node) GetNodeType() Node_Type {
	if x != nil {
		return x.NodeType
	}
	return Node_TypeLogicalExpression
}

func (x *Node) GetChildren() []*Node {
	if x != nil {
		return x.Children
	}
	return nil
}

func (m *Node) GetValue() isNode_Value {
	if m != nil {
		return m.Value
	}
	return nil
}

func (x *Node) GetStringValue() string {
	if x, ok := x.GetValue().(*Node_StringValue); ok {
		return x.StringValue
	}
	return ""
}

func (x *Node) GetBooleanValue() bool {
	if x, ok := x.GetValue().(*Node_BooleanValue); ok {
		return x.BooleanValue
	}
	return false
}

func (x *Node) GetIntegerValue() int64 {
	if x, ok := x.GetValue().(*Node_IntegerValue); ok {
		return x.IntegerValue
	}
	return 0
}

func (x *Node) GetUnsignedValue() uint64 {
	if x, ok := x.GetValue().(*Node_UnsignedValue); ok {
		return x.UnsignedValue
	}
	return 0
}

func (x *Node) GetFloatValue() float64 {
	if x, ok := x.GetValue().(*Node_FloatValue); ok {
		return x.FloatValue
	}
	return 0
}

func (x *Node) GetRegexValue() string {
	if x, ok := x.GetValue().(*Node_RegexValue); ok {
		return x.RegexValue
	}
	return ""
}

func (x *Node) GetTagRefValue() string {
	if x, ok := x.GetValue().(*Node_TagRefValue); ok {
		return x.TagRefValue
	}
	return ""
}

func (x *Node) GetFieldRefValue() string {
	if x, ok := x.GetValue().(*Node_FieldRefValue); ok {
		return x.FieldRefValue
	}
	return ""
}

func (x *Node) GetLogical() Node_Logical {
	if x, ok := x.GetValue().(*Node_Logical_); ok {
		return x.Logical
	}
	return Node_LogicalAnd
}

func (x *Node) GetComparison() Node_Comparison {
	if x, ok := x.GetValue().(*Node_Comparison_); ok {
		return x.Comparison
	}
	return Node_ComparisonEqual
}

type isNode_Value interface {
	isNode_Value()
}

type Node_StringValue struct {
	StringValue string `protobuf:"bytes,3,opt,name=StringValue,proto3,oneof"`
}

type Node_BooleanValue struct {
	BooleanValue bool `protobuf:"varint,4,opt,name=BooleanValue,proto3,oneof"`
}

type Node_IntegerValue struct {
	IntegerValue int64 `protobuf:"varint,5,opt,name=IntegerValue,proto3,oneof"`
}

type Node_UnsignedValue struct {
	UnsignedValue uint64 `protobuf:"varint,6,opt,name=UnsignedValue,proto3,oneof"`
}

type Node_FloatValue struct {
	FloatValue float64 `protobuf:"fixed64,7,opt,name=FloatValue,proto3,oneof"`
}

type Node_RegexValue struct {
	RegexValue string `protobuf:"bytes,8,opt,name=RegexValue,proto3,oneof"`
}

type Node_TagRefValue struct {
	TagRefValue string `protobuf:"bytes,9,opt,name=TagRefValue,proto3,oneof"`
}

type Node_FieldRefValue struct {
	FieldRefValue string `protobuf:"bytes,10,opt,name=FieldRefValue,proto3,oneof"`
}

type Node_Logical_ struct {
	Logical Node_Logical `protobuf:"varint,11,opt,name=logical,proto3,enum=influxdata.platform.storage.Node_Logical,oneof"`
}

type Node_Comparison_ struct {
	Comparison Node_Comparison `protobuf:"varint,12,opt,name=comparison,proto3,enum=influxdata.platform.storage.Node_Comparison,oneof"`
}

func (*Node_StringValue) isNode_Value() {}

func (*Node_BooleanValue) isNode_Value() {}

func (*Node_IntegerValue) isNode_Value() {}

func (*Node_UnsignedValue) isNode_Value() {}

func (*Node_FloatValue) isNode_Value() {}

func (*Node_RegexValue) isNode_Value() {}

func (*Node_TagRefValue) isNode_Value() {}

func (*Node_FieldRefValue) isNode_Value() {}

func (*Node_Logical_) isNode_Value() {}

func (*Node_Comparison_) isNode_Value() {}

type Predicate struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	Root *Node `protobuf:"bytes,1,opt,name=root,proto3" json:"root,omitempty"`
}

func (x *Predicate) Reset() {
	*x = Predicate{}
	if protoimpl.UnsafeEnabled {
		mi := &file_predicate_proto_msgTypes[1]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *Predicate) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*Predicate) ProtoMessage() {}

func (x *Predicate) ProtoReflect() protoreflect.Message {
	mi := &file_predicate_proto_msgTypes[1]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use Predicate.ProtoReflect.Descriptor instead.
func (*Predicate) Descriptor() ([]byte, []int) {
	return file_predicate_proto_rawDescGZIP(), []int{1}
}

func (x *Predicate) GetRoot() *Node {
	if x != nil {
		return x.Root
	}
	return nil
}

var File_predicate_proto protoreflect.FileDescriptor

var file_predicate_proto_rawDesc = []byte{
	0x0a, 0x0f, 0x70, 0x72, 0x65, 0x64, 0x69, 0x63, 0x61, 0x74, 0x65, 0x2e, 0x70, 0x72, 0x6f, 0x74,
	0x6f, 0x12, 0x1b, 0x69, 0x6e, 0x66, 0x6c, 0x75, 0x78, 0x64, 0x61, 0x74, 0x61, 0x2e, 0x70, 0x6c,
	0x61, 0x74, 0x66, 0x6f, 0x72, 0x6d, 0x2e, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x22, 0xed,
	0x07, 0x0a, 0x04, 0x4e, 0x6f, 0x64, 0x65, 0x12, 0x43, 0x0a, 0x09, 0x6e, 0x6f, 0x64, 0x65, 0x5f,
	0x74, 0x79, 0x70, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0e, 0x32, 0x26, 0x2e, 0x69, 0x6e, 0x66,
	0x6c, 0x75, 0x78, 0x64, 0x61, 0x74, 0x61, 0x2e, 0x70, 0x6c, 0x61, 0x74, 0x66, 0x6f, 0x72, 0x6d,
	0x2e, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x2e, 0x4e, 0x6f, 0x64, 0x65, 0x2e, 0x54, 0x79,
	0x70, 0x65, 0x52, 0x08, 0x6e, 0x6f, 0x64, 0x65, 0x54, 0x79, 0x70, 0x65, 0x12, 0x3d, 0x0a, 0x08,
	0x63, 0x68, 0x69, 0x6c, 0x64, 0x72, 0x65, 0x6e, 0x18, 0x02, 0x20, 0x03, 0x28, 0x0b, 0x32, 0x21,
	0x2e, 0x69, 0x6e, 0x66, 0x6c, 0x75, 0x78, 0x64, 0x61, 0x74, 0x61, 0x2e, 0x70, 0x6c, 0x61, 0x74,
	0x66, 0x6f, 0x72, 0x6d, 0x2e, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x2e, 0x4e, 0x6f, 0x64,
	0x65, 0x52, 0x08, 0x63, 0x68, 0x69, 0x6c, 0x64, 0x72, 0x65, 0x6e, 0x12, 0x22, 0x0a, 0x0b, 0x53,
	0x74, 0x72, 0x69, 0x6e, 0x67, 0x56, 0x61, 0x6c, 0x75, 0x65, 0x18, 0x03, 0x20, 0x01, 0x28, 0x09,
	0x48, 0x00, 0x52, 0x0b, 0x53, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x56, 0x61, 0x6c, 0x75, 0x65, 0x12,
	0x24, 0x0a, 0x0c, 0x42, 0x6f, 0x6f, 0x6c, 0x65, 0x61, 0x6e, 0x56, 0x61, 0x6c, 0x75, 0x65, 0x18,
	0x04, 0x20, 0x01, 0x28, 0x08, 0x48, 0x00, 0x52, 0x0c, 0x42, 0x6f, 0x6f, 0x6c, 0x65, 0x61, 0x6e,
	0x56, 0x61, 0x6c, 0x75, 0x65, 0x12, 0x24, 0x0a, 0x0c, 0x49, 0x6e, 0x74, 0x65, 0x67, 0x65, 0x72,
	0x56, 0x61, 0x6c, 0x75, 0x65, 0x18, 0x05, 0x20, 0x01, 0x28, 0x03, 0x48, 0x00, 0x52, 0x0c, 0x49,
	0x6e, 0x74, 0x65, 0x67, 0x65, 0x72, 0x56, 0x61, 0x6c, 0x75, 0x65, 0x12, 0x26, 0x0a, 0x0d, 0x55,
	0x6e, 0x73, 0x69, 0x67, 0x6e, 0x65, 0x64, 0x56, 0x61, 0x6c, 0x75, 0x65, 0x18, 0x06, 0x20, 0x01,
	0x28, 0x04, 0x48, 0x00, 0x52, 0x0d, 0x55, 0x6e, 0x73, 0x69, 0x67, 0x6e, 0x65, 0x64, 0x56, 0x61,
	0x6c, 0x75, 0x65, 0x12, 0x20, 0x0a, 0x0a, 0x46, 0x6c, 0x6f, 0x61, 0x74, 0x56, 0x61, 0x6c, 0x75,
	0x65, 0x18, 0x07, 0x20, 0x01, 0x28, 0x01, 0x48, 0x00, 0x52, 0x0a, 0x46, 0x6c, 0x6f, 0x61, 0x74,
	0x56, 0x61, 0x6c, 0x75, 0x65, 0x12, 0x20, 0x0a, 0x0a, 0x52, 0x65, 0x67, 0x65, 0x78, 0x56, 0x61,
	0x6c, 0x75, 0x65, 0x18, 0x08, 0x20, 0x01, 0x28, 0x09, 0x48, 0x00, 0x52, 0x0a, 0x52, 0x65, 0x67,
	0x65, 0x78, 0x56, 0x61, 0x6c, 0x75, 0x65, 0x12, 0x22, 0x0a, 0x0b, 0x54, 0x61, 0x67, 0x52, 0x65,
	0x66, 0x56, 0x61, 0x6c, 0x75, 0x65, 0x18, 0x09, 0x20, 0x01, 0x28, 0x09, 0x48, 0x00, 0x52, 0x0b,
	0x54, 0x61, 0x67, 0x52, 0x65, 0x66, 0x56, 0x61, 0x6c, 0x75, 0x65, 0x12, 0x26, 0x0a, 0x0d, 0x46,
	0x69, 0x65, 0x6c, 0x64, 0x52, 0x65, 0x66, 0x56, 0x61, 0x6c, 0x75, 0x65, 0x18, 0x0a, 0x20, 0x01,
	0x28, 0x09, 0x48, 0x00, 0x52, 0x0d, 0x46, 0x69, 0x65, 0x6c, 0x64, 0x52, 0x65, 0x66, 0x56, 0x61,
	0x6c, 0x75, 0x65, 0x12, 0x45, 0x0a, 0x07, 0x6c, 0x6f, 0x67, 0x69, 0x63, 0x61, 0x6c, 0x18, 0x0b,
	0x20, 0x01, 0x28, 0x0e, 0x32, 0x29, 0x2e, 0x69, 0x6e, 0x66, 0x6c, 0x75, 0x78, 0x64, 0x61, 0x74,
	0x61, 0x2e, 0x70, 0x6c, 0x61, 0x74, 0x66, 0x6f, 0x72, 0x6d, 0x2e, 0x73, 0x74, 0x6f, 0x72, 0x61,
	0x67, 0x65, 0x2e, 0x4e, 0x6f, 0x64, 0x65, 0x2e, 0x4c, 0x6f, 0x67, 0x69, 0x63, 0x61, 0x6c, 0x48,
	0x00, 0x52, 0x07, 0x6c, 0x6f, 0x67, 0x69, 0x63, 0x61, 0x6c, 0x12, 0x4e, 0x0a, 0x0a, 0x63, 0x6f,
	0x6d, 0x70, 0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e, 0x18, 0x0c, 0x20, 0x01, 0x28, 0x0e, 0x32, 0x2c,
	0x2e, 0x69, 0x6e, 0x66, 0x6c, 0x75, 0x78, 0x64, 0x61, 0x74, 0x61, 0x2e, 0x70, 0x6c, 0x61, 0x74,
	0x66, 0x6f, 0x72, 0x6d, 0x2e, 0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x2e, 0x4e, 0x6f, 0x64,
	0x65, 0x2e, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e, 0x48, 0x00, 0x52, 0x0a,
	0x63, 0x6f, 0x6d, 0x70, 0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e, 0x22, 0x8b, 0x01, 0x0a, 0x04, 0x54,
	0x79, 0x70, 0x65, 0x12, 0x19, 0x0a, 0x15, 0x54, 0x79, 0x70, 0x65, 0x4c, 0x6f, 0x67, 0x69, 0x63,
	0x61, 0x6c, 0x45, 0x78, 0x70, 0x72, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x10, 0x00, 0x12, 0x1c,
	0x0a, 0x18, 0x54, 0x79, 0x70, 0x65, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e,
	0x45, 0x78, 0x70, 0x72, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x10, 0x01, 0x12, 0x17, 0x0a, 0x13,
	0x54, 0x79, 0x70, 0x65, 0x50, 0x61, 0x72, 0x65, 0x6e, 0x45, 0x78, 0x70, 0x72, 0x65, 0x73, 0x73,
	0x69, 0x6f, 0x6e, 0x10, 0x02, 0x12, 0x0e, 0x0a, 0x0a, 0x54, 0x79, 0x70, 0x65, 0x54, 0x61, 0x67,
	0x52, 0x65, 0x66, 0x10, 0x03, 0x12, 0x0f, 0x0a, 0x0b, 0x54, 0x79, 0x70, 0x65, 0x4c, 0x69, 0x74,
	0x65, 0x72, 0x61, 0x6c, 0x10, 0x04, 0x12, 0x10, 0x0a, 0x0c, 0x54, 0x79, 0x70, 0x65, 0x46, 0x69,
	0x65, 0x6c, 0x64, 0x52, 0x65, 0x66, 0x10, 0x05, 0x22, 0xe0, 0x01, 0x0a, 0x0a, 0x43, 0x6f, 0x6d,
	0x70, 0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e, 0x12, 0x13, 0x0a, 0x0f, 0x43, 0x6f, 0x6d, 0x70, 0x61,
	0x72, 0x69, 0x73, 0x6f, 0x6e, 0x45, 0x71, 0x75, 0x61, 0x6c, 0x10, 0x00, 0x12, 0x16, 0x0a, 0x12,
	0x43, 0x6f, 0x6d, 0x70, 0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e, 0x4e, 0x6f, 0x74, 0x45, 0x71, 0x75,
	0x61, 0x6c, 0x10, 0x01, 0x12, 0x18, 0x0a, 0x14, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x72, 0x69, 0x73,
	0x6f, 0x6e, 0x53, 0x74, 0x61, 0x72, 0x74, 0x73, 0x57, 0x69, 0x74, 0x68, 0x10, 0x02, 0x12, 0x13,
	0x0a, 0x0f, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e, 0x52, 0x65, 0x67, 0x65,
	0x78, 0x10, 0x03, 0x12, 0x16, 0x0a, 0x12, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x72, 0x69, 0x73, 0x6f,
	0x6e, 0x4e, 0x6f, 0x74, 0x52, 0x65, 0x67, 0x65, 0x78, 0x10, 0x04, 0x12, 0x12, 0x0a, 0x0e, 0x43,
	0x6f, 0x6d, 0x70, 0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e, 0x4c, 0x65, 0x73, 0x73, 0x10, 0x05, 0x12,
	0x17, 0x0a, 0x13, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e, 0x4c, 0x65, 0x73,
	0x73, 0x45, 0x71, 0x75, 0x61, 0x6c, 0x10, 0x06, 0x12, 0x15, 0x0a, 0x11, 0x43, 0x6f, 0x6d, 0x70,
	0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e, 0x47, 0x72, 0x65, 0x61, 0x74, 0x65, 0x72, 0x10, 0x07, 0x12,
	0x1a, 0x0a, 0x16, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x72, 0x69, 0x73, 0x6f, 0x6e, 0x47, 0x72, 0x65,
	0x61, 0x74, 0x65, 0x72, 0x45, 0x71, 0x75, 0x61, 0x6c, 0x10, 0x08, 0x22, 0x28, 0x0a, 0x07, 0x4c,
	0x6f, 0x67, 0x69, 0x63, 0x61, 0x6c, 0x12, 0x0e, 0x0a, 0x0a, 0x4c, 0x6f, 0x67, 0x69, 0x63, 0x61,
	0x6c, 0x41, 0x6e, 0x64, 0x10, 0x00, 0x12, 0x0d, 0x0a, 0x09, 0x4c, 0x6f, 0x67, 0x69, 0x63, 0x61,
	0x6c, 0x4f, 0x72, 0x10, 0x01, 0x42, 0x07, 0x0a, 0x05, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x22, 0x42,
	0x0a, 0x09, 0x50, 0x72, 0x65, 0x64, 0x69, 0x63, 0x61, 0x74, 0x65, 0x12, 0x35, 0x0a, 0x04, 0x72,
	0x6f, 0x6f, 0x74, 0x18, 0x01, 0x20, 0x01, 0x28, 0x0b, 0x32, 0x21, 0x2e, 0x69, 0x6e, 0x66, 0x6c,
	0x75, 0x78, 0x64, 0x61, 0x74, 0x61, 0x2e, 0x70, 0x6c, 0x61, 0x74, 0x66, 0x6f, 0x72, 0x6d, 0x2e,
	0x73, 0x74, 0x6f, 0x72, 0x61, 0x67, 0x65, 0x2e, 0x4e, 0x6f, 0x64, 0x65, 0x52, 0x04, 0x72, 0x6f,
	0x6f, 0x74, 0x42, 0x0d, 0x5a, 0x0b, 0x2e, 0x3b, 0x64, 0x61, 0x74, 0x61, 0x74, 0x79, 0x70, 0x65,
	0x73, 0x62, 0x06, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x33,
}

var (
	file_predicate_proto_rawDescOnce sync.Once
	file_predicate_proto_rawDescData = file_predicate_proto_rawDesc
)

func file_predicate_proto_rawDescGZIP() []byte {
	file_predicate_proto_rawDescOnce.Do(func() {
		file_predicate_proto_rawDescData = protoimpl.X.CompressGZIP(file_predicate_proto_rawDescData)
	})
	return file_predicate_proto_rawDescData
}

var file_predicate_proto_enumTypes = make([]protoimpl.EnumInfo, 3)
var file_predicate_proto_msgTypes = make([]protoimpl.MessageInfo, 2)
var file_predicate_proto_goTypes = []interface{}{
	(Node_Type)(0),       // 0: influxdata.platform.storage.Node.Type
	(Node_Comparison)(0), // 1: influxdata.platform.storage.Node.Comparison
	(Node_Logical)(0),    // 2: influxdata.platform.storage.Node.Logical
	(*Node)(nil),         // 3: influxdata.platform.storage.Node
	(*Predicate)(nil),    // 4: influxdata.platform.storage.Predicate
}
var file_predicate_proto_depIdxs = []int32{
	0, // 0: influxdata.platform.storage.Node.node_type:type_name -> influxdata.platform.storage.Node.Type
	3, // 1: influxdata.platform.storage.Node.children:type_name -> influxdata.platform.storage.Node
	2, // 2: influxdata.platform.storage.Node.logical:type_name -> influxdata.platform.storage.Node.Logical
	1, // 3: influxdata.platform.storage.Node.comparison:type_name -> influxdata.platform.storage.Node.Comparison
	3, // 4: influxdata.platform.storage.Predicate.root:type_name -> influxdata.platform.storage.Node
	5, // [5:5] is the sub-list for method output_type
	5, // [5:5] is the sub-list for method input_type
	5, // [5:5] is the sub-list for extension type_name
	5, // [5:5] is the sub-list for extension extendee
	0, // [0:5] is the sub-list for field type_name
}

func init() { file_predicate_proto_init() }
func file_predicate_proto_init() {
	if File_predicate_proto != nil {
		return
	}
	if !protoimpl.UnsafeEnabled {
		file_predicate_proto_msgTypes[0].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*Node); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
		file_predicate_proto_msgTypes[1].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*Predicate); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
	}
	file_predicate_proto_msgTypes[0].OneofWrappers = []interface{}{
		(*Node_StringValue)(nil),
		(*Node_BooleanValue)(nil),
		(*Node_IntegerValue)(nil),
		(*Node_UnsignedValue)(nil),
		(*Node_FloatValue)(nil),
		(*Node_RegexValue)(nil),
		(*Node_TagRefValue)(nil),
		(*Node_FieldRefValue)(nil),
		(*Node_Logical_)(nil),
		(*Node_Comparison_)(nil),
	}
	type x struct{}
	out := protoimpl.TypeBuilder{
		File: protoimpl.DescBuilder{
			GoPackagePath: reflect.TypeOf(x{}).PkgPath(),
			RawDescriptor: file_predicate_proto_rawDesc,
			NumEnums:      3,
			NumMessages:   2,
			NumExtensions: 0,
			NumServices:   0,
		},
		GoTypes:           file_predicate_proto_goTypes,
		DependencyIndexes: file_predicate_proto_depIdxs,
		EnumInfos:         file_predicate_proto_enumTypes,
		MessageInfos:      file_predicate_proto_msgTypes,
	}.Build()
	File_predicate_proto = out.File
	file_predicate_proto_rawDesc = nil
	file_predicate_proto_goTypes = nil
	file_predicate_proto_depIdxs = nil
}
