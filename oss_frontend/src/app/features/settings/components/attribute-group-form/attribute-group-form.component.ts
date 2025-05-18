import { Component, OnInit } from '@angular/core';
import {FormBuilder, FormGroup, ReactiveFormsModule, Validators} from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import {TranslatePipe, TranslateService} from '@ngx-translate/core';
import { AttributeGroupService } from '../../../../shared/services/attribute-group.service';
import { RadiusAttributeService } from '../../../../shared/services/radius-attribute.service';
import { RadiusAttribute, AttributeType } from '../../../../shared/models/radius-attribute.model';

@Component({
  selector: 'app-attribute-group-form',
  templateUrl: './attribute-group-form.component.html',
  imports: [
    TranslatePipe,
    ReactiveFormsModule,
  ],
  styleUrls: ['./attribute-group-form.component.scss'],
})
export class AttributeGroupFormComponent implements OnInit {
  groupForm: FormGroup;
  attributeForm: FormGroup;
  isEditMode = false;
  loading = false;
  submitting = false;
  error: string | null = null;
  attributes: RadiusAttribute[] = [];
  attributeTypes: AttributeType[] = ['string', 'integer', 'ipaddr', 'date', 'octets'];

  constructor(
    private fb: FormBuilder,
    private attributeGroupService: AttributeGroupService,
    private radiusAttributeService: RadiusAttributeService,
    private route: ActivatedRoute,
    private router: Router,
    private translate: TranslateService
  ) {
    this.groupForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      description: ['']
    });

    this.attributeForm = this.fb.group({
      vendor_id: [0, [Validators.required, Validators.min(0)]],
      attribute_id: ['', [Validators.required, Validators.min(1)]],
      attribute_name: ['', [Validators.required, Validators.maxLength(255)]],
      attribute_type: ['string', Validators.required],
      attribute_value: ['', [Validators.required, Validators.maxLength(255)]]
    });
  }

  ngOnInit(): void {
    const id = this.route.snapshot.paramMap.get('id');
    if (id) {
      this.isEditMode = true;
      this.loadGroup(parseInt(id));
    }
  }

  loadGroup(id: number): void {
    this.loading = true;
    this.attributeGroupService.getGroup(id).subscribe({
      next: (group) => {
        this.groupForm.patchValue({
          name: group.name,
          description: group.description
        });
        this.loadAttributes(id);
        this.loading = false;
      },
      error: (error) => {
        this.error = this.translate.instant('settings.attributeGroups.errors.loadFailed');
        this.loading = false;
      }
    });
  }

  loadAttributes(groupId: number): void {
    this.radiusAttributeService.getAttributesByGroup(groupId).subscribe({
      next: (attributes) => {
        this.attributes = attributes;
      },
      error: (error) => {
        this.error = this.translate.instant('settings.attributeGroups.errors.loadFailed');
      }
    });
  }

  onSubmit(): void {
    if (this.groupForm.valid) {
      this.submitting = true;
      this.error = null;

      const groupData = this.groupForm.value;
      const id = this.route.snapshot.paramMap.get('id');

      const request = id
        ? this.attributeGroupService.updateGroup(parseInt(id), groupData)
        : this.attributeGroupService.createGroup(groupData);

      request.subscribe({
        next: (group) => {
          if (!id) {
            if (group && group.id) {
              this.router.navigate(['/settings/attribute-groups', group.id, 'edit']);
            } else {
              this.router.navigate(['/settings/attribute-groups']);
            }
          }
          this.submitting = false;
        },
        error: (error) => {
          this.error = this.translate.instant('settings.attributeGroups.errors.saveFailed');
          this.submitting = false;
        }
      });
    }
  }

  onAddAttribute(): void {
    if (this.attributeForm.valid) {
      const groupId = this.route.snapshot.paramMap.get('id');
      if (!groupId) {
        this.error = this.translate.instant('settings.attributeGroups.radiusAttributes.saveGroupFirst');
        return;
      }

      const attributeData = {
        ...this.attributeForm.value,
        group: parseInt(groupId)
      };

      this.radiusAttributeService.createAttribute(attributeData).subscribe({
        next: (attribute) => {
          this.attributes.push(attribute);
          this.attributeForm.reset({
            vendor_id: 0,
            attribute_type: 'string'
          });
        },
        error: (error) => {
          this.error = this.translate.instant('settings.attributeGroups.errors.addAttributeFailed');
        }
      });
    }
  }

  onDeleteAttribute(id: number): void {
    if (confirm(this.translate.instant('common.confirmDelete'))) {
      this.radiusAttributeService.deleteAttribute(id).subscribe({
        next: () => {
          this.attributes = this.attributes.filter(attr => attr.id !== id);
        },
        error: (error) => {
          this.error = this.translate.instant('settings.attributeGroups.errors.deleteAttributeFailed');
        }
      });
    }
  }

  onCancel(): void {
    this.router.navigate(['/settings/attribute-groups']);
  }

  isFieldInvalid(fieldName: string): boolean {
    const field = this.groupForm.get(fieldName);
    return field ? field.invalid && (field.dirty || field.touched) : false;
  }

  getErrorMessage(fieldName: string): string {
    const field = this.groupForm.get(fieldName);
    if (!field) return '';

    if (field.errors?.['required']) {
      return this.translate.instant('common.errors.required', { field: fieldName });
    }
    if (field.errors?.['maxlength']) {
      return this.translate.instant('common.errors.maxLength', { field: fieldName, length: 255 });
    }
    return '';
  }
}
