import { Component, OnInit, ChangeDetectorRef } from '@angular/core';
import {FormBuilder, FormGroup, FormsModule, ReactiveFormsModule, Validators, FormArray, AbstractControl} from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { forkJoin, Observable, of } from 'rxjs';
import { catchError, finalize, switchMap } from 'rxjs/operators';
import {TranslatePipe, TranslateService} from '@ngx-translate/core';
import { CommonModule } from '@angular/common';
import { UserGroup } from "../../../../shared/models/user-group.model";
import { UserIdentifierType } from "../../../../shared/models/user-identifier-type.model";
import { UserIdentifier } from "../../../../shared/models/user-identifier.model";
import { UserService } from "../../../../shared/services/user.service";
import { UserIdentifierTypeService } from "../../../../shared/services/user-identifier-type.service";
import { AttributeGroup } from 'src/app/shared/models/attribute-group.model';
import { AttributeGroupService } from 'src/app/shared/services/attribute-group.service';

interface LoadDataResult {
  groups: UserGroup[];
  identifierTypes: UserIdentifierType[];
  authAttributeGroups: AttributeGroup[];
}

interface UserResponse {
  email: string;
  external_id: string;
  first_name: string;
  last_name: string;
  phone_number: string;
  is_active: boolean;
  groups: UserGroup[];
  identifiers: UserIdentifier[];
  allow_any_nas?: boolean;
}

interface EditModeResult extends LoadDataResult {
  user: UserResponse;
}

@Component({
  selector: 'app-user-form',
  templateUrl: './user-form.component.html',
  imports: [
    TranslatePipe,
    ReactiveFormsModule,
    FormsModule,
    CommonModule
  ],
  styleUrls: ['./user-form.component.scss']
})
export class UserFormComponent implements OnInit {
  userForm: FormGroup;
  identifiersForm: FormArray;
  isEditMode = false;
  userId: number | null = null;
  userGroups: UserGroup[] = [];
  identifierTypes: UserIdentifierType[] = [];
  authAttributeGroups: AttributeGroup[] = [];
  identifiers: Partial<UserIdentifier>[] = [];
  loading = true;
  submitting = false;
  error: string | null = null;

  constructor(
    private fb: FormBuilder,
    private userService: UserService,
    private identifierTypeService: UserIdentifierTypeService,
    private authAttributeGroupService: AttributeGroupService,
    private route: ActivatedRoute,
    private router: Router,
    private translate: TranslateService,
  ) {
    this.userForm = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      first_name: [''],
      last_name: [''],
      external_id: [null],
      phone_number: [''],
      is_active: [true],
      allow_any_nas: [null],
      group_ids: [[]],
      identifiers: this.fb.array([])
    });
    this.identifiersForm = this.userForm.get('identifiers') as FormArray;
  }

  ngOnInit(): void {
    this.loading = true;
    this.error = null;

    const loadData$: Observable<LoadDataResult> = forkJoin({
      groups: this.userService.getUserGroupList(),
      identifierTypes: this.identifierTypeService.getIdentifierTypes(),
      authAttributeGroups: this.authAttributeGroupService.getAllGroupsList()
    });

    this.route.params.pipe(
      switchMap(params => {
        if (params['id']) {
          this.isEditMode = true;
          this.userId = +params['id'];
          return forkJoin({
            user: this.userService.getUser(this.userId),
            groups: this.userService.getUserGroupList(),
            identifierTypes: this.identifierTypeService.getIdentifierTypes(),
            authAttributeGroups: this.authAttributeGroupService.getAllGroupsList()
          }) as Observable<EditModeResult>;
        }
        return loadData$;
      }),
      catchError(error => {
        this.error = error.message || 'An error occurred while loading data';
        this.loading = false;
        return of(null);
      })
    ).subscribe({
      next: (result) => {

        if (result) {
          this.authAttributeGroups = result.authAttributeGroups;

          if (this.isEditMode && 'user' in result) {
            const editResult = result as EditModeResult;
            const userData = {
              email: editResult.user.email,
              first_name: editResult.user.first_name,
              last_name: editResult.user.last_name,
              phone_number: editResult.user.phone_number,
              is_active: editResult.user.is_active,
              group_ids: editResult.user.groups.map(g => g.id)
            };
            this.userForm.patchValue(userData);

            // Clear existing form array
            while (this.identifiersForm.length) {
              this.identifiersForm.removeAt(0);
            }

            // Create form controls for existing identifiers
            if (editResult.user.identifiers) {
              console.log("auth_attribute_group", this.authAttributeGroups);
              editResult.user.identifiers.forEach(identifier => {
                const identifierForm = this.createIdentifierForm();
                identifierForm.patchValue({
                  identifier_type_id: identifier.identifier_type.id,
                  value: identifier.value,
                  is_enabled: identifier.is_enabled,
                  reject_expired: identifier.reject_expired,
                  auth_attribute_group: identifier.auth_attribute_group?.id,
                  expired_auth_attribute_group: identifier.expired_auth_attribute_group?.id,
                  expiration_date: identifier.expiration_date,
                  comment: identifier.comment,
                  plain_password: identifier.plain_password
                });

                console.log('Selected identifier expired_auth_attribute_group is %s', identifier.auth_attribute_group?.id)

                // Handle disabled state for expired_auth_attribute_group
                if (identifier.reject_expired) {
                  identifierForm.get('expired_auth_attribute_group')?.disable();
                }

                this.identifiersForm.push(identifierForm);
              });
            }
          }
          this.userGroups = result.groups;
          this.identifierTypes = result.identifierTypes;
        }
        this.loading = false;
      },
      error: (error) => {
        this.error = error.message || 'An error occurred while loading data';
        this.loading = false;
      }
    });
  }

  onGroupChange(event: Event, group: UserGroup): void {
    const checkbox = event.target as HTMLInputElement;
    const groups = this.userForm.get('group_ids')?.value || [];

    if (checkbox.checked) {
      groups.push(group.id);
    } else {
      const index = groups.indexOf(group.id);
      if (index > -1) {
        groups.splice(index, 1);
      }
    }

    this.userForm.patchValue({ group_ids: groups });
  }

  onIdentifierTypeChange(identifier: Partial<UserIdentifier>): void {
    // Clear password when changing type
    identifier.plain_password = '';
  }

  getIdentifierTypeCode(index: number): string | undefined {
    const typeId = this.identifiersForm.at(index).get('identifier_type_id')?.value;
    const type = this.identifierTypes.find(t => t.id === typeId);
    return type?.code;
  }

  shouldShowPasswordField(index: number): boolean {
    const typeCode = this.getIdentifierTypeCode(index);
    return ['PWD', 'MAC'].includes(typeCode || '');
  }

  getValuePlaceholder(index: number): string {
    const typeCode = this.getIdentifierTypeCode(index);
    switch (typeCode) {
      case 'PWD':
        return 'Username';
      case 'SIM':
        return 'SIM Number';
      case 'MAC':
        return 'MAC Address';
      default:
        return 'Value';
    }
  }

  getPasswordPlaceholder(index: number): string {
    const typeCode = this.getIdentifierTypeCode(index);
    switch (typeCode) {
      case 'PWD':
        return 'Password';
      case 'MAC':
        return 'MAC Password';
      default:
        return 'Password';
    }
  }

  createIdentifierForm(): FormGroup {
    return this.fb.group({
      identifier_type_id: [null, Validators.required],
      value: ['', Validators.required],
      plain_password: [''],
      is_enabled: [true],
      reject_expired: [false],
      auth_attribute_group: [null],
      expired_auth_attribute_group: [{value: null, disabled: false}],
      expiration_date: [null],
      comment: ['']
    });
  }

  addIdentifier(): void {
    this.identifiersForm.push(this.createIdentifierForm());
  }

  removeIdentifier(index: number): void {
    this.identifiersForm.removeAt(index);
  }

  onRejectExpiredChange(index: number): void {
    const identifierForm = this.identifiersForm.at(index);
    const rejectExpired = identifierForm.get('reject_expired')?.value;
    const expiredAuthGroupControl = identifierForm.get('expired_auth_attribute_group');

    if (rejectExpired) {
      expiredAuthGroupControl?.setValue(null);
      expiredAuthGroupControl?.disable();
    } else {
      expiredAuthGroupControl?.enable();
    }
  }

  hasValidIdentifiers(): boolean {
    return this.identifiersForm.length > 0 && this.identifiersForm.valid;
  }

  getIdentifiersErrorMessage(): string {
    if (this.identifiersForm.length === 0) {
      return 'At least one identifier is required';
    }
    if (!this.identifiersForm.valid) {
      return 'Please fill in all required fields for identifiers';
    }
    return '';
  }

  onSubmit(): void {
    if (this.userForm.invalid || !this.hasValidIdentifiers()) {
      return;
    }

    this.submitting = true;
    this.error = null;

    // Get the raw form values
    const formValue = this.userForm.getRawValue();

    // Format identifiers data
    const identifiers = this.identifiersForm.controls.map(control => {
      const identifierValue = control.getRawValue();
      // Get the original identifier from the identifiers array if it exists
      const originalIdentifier = this.identifiers.find(i =>
        i.identifier_type?.id === identifierValue.identifier_type_id &&
        i.value === identifierValue.value
      );

      return {
        id: originalIdentifier?.id,
        identifier_type_id: identifierValue.identifier_type_id,
        value: identifierValue.value,
        plain_password: identifierValue.plain_password,
        is_enabled: identifierValue.is_enabled,
        reject_expired: identifierValue.reject_expired,
        auth_attribute_group_id: identifierValue.auth_attribute_group || null,
        expired_auth_attribute_group_id: identifierValue.expired_auth_attribute_group || null,
        expiration_date: identifierValue.expiration_date,
        comment: identifierValue.comment
      };
    });

    const userData = {
      ...formValue,
      group_ids: formValue.group_ids || [],
      identifiers: identifiers
    };

    const request$ = this.isEditMode
      ? this.userService.updateUser(this.userId!, userData)
      : this.userService.createUser(userData);

    request$.pipe(
      catchError(error => {
        this.error = error.message || 'An error occurred while saving the user';
        return of(null);
      }),
      finalize(() => {
        this.submitting = false;
      })
    ).subscribe(result => {
      if (result) {
        this.router.navigate(['/users/users']);
      }
    });
  }

  goBack(): void {
    this.router.navigate(['/users/users']);
  }

  getIdentifierFormGroup(control: AbstractControl): FormGroup {
    return control as FormGroup;
  }
}
